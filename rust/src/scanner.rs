use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rayon::prelude::*;
use sysinfo::Disks;

use crate::models::{sort_items_default, ItemType, ScanEvent, ScanItem, ScanResult};

pub fn parse_size_limit(text: &str) -> f64 {
    let text = text.trim();
    if text.is_empty() {
        return 5000.0;
    }

    let upper = text.to_uppercase();
    let multiplier: f64 = if upper.contains("TB") {
        1024.0 * 1024.0
    } else if upper.contains("GB") {
        1024.0
    } else if upper.contains("KB") {
        1.0 / 1024.0
    } else {
        // Default to MB
        1.0
    };

    let numeric_chars: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .map(|c| if c == ',' { '.' } else { c })
        .collect();

    match numeric_chars.parse::<f64>() {
        Ok(val) => val * multiplier,
        Err(_) => 5000.0,
    }
}

pub fn get_disk_info(target_path: &Path) -> (u64, u64) {
    let disks = Disks::new_with_refreshed_list();
    let mut best_match: Option<(usize, u64, u64)> = None;

    let target_str = target_path.to_string_lossy();

    for disk in disks.list() {
        let mount_str = disk.mount_point().to_string_lossy();
        if target_str.starts_with(mount_str.as_ref()) {
            let match_len = mount_str.len();
            if best_match.as_ref().map_or(true, |(len, _, _)| match_len > *len) {
                best_match = Some((match_len, disk.available_space(), disk.total_space()));
            }
        }
    }

    if let Some((_, free, total)) = best_match {
        (free, total)
    } else if let Some(first) = disks.list().first() {
        (first.available_space(), first.total_space())
    } else {
        (0, 0)
    }
}

pub fn should_skip_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    #[cfg(target_os = "linux")]
    {
        if s.starts_with("/proc")
            || s.starts_with("/sys")
            || s.starts_with("/dev")
            || s.starts_with("/run")
            || s.contains("/.gvfs")
        {
            return true;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if s.contains(".sys")
            || s.contains("$Recycle.Bin")
            || s.contains("System Volume Information")
        {
            return true;
        }
    }
    false
}

/// Monitor de progreso concurrente y libre de bloqueos (lock-free) para hilos paralelos
#[derive(Clone)]
pub struct ScanReporter {
    sender: Sender<ScanEvent>,
    pub files_scanned: Arc<AtomicUsize>,
    pub bytes_scanned: Arc<AtomicU64>,
    last_notify_ms: Arc<AtomicU64>,
    start_time: Instant,
}

impl ScanReporter {
    pub fn new(sender: Sender<ScanEvent>) -> Self {
        Self {
            sender,
            files_scanned: Arc::new(AtomicUsize::new(0)),
            bytes_scanned: Arc::new(AtomicU64::new(0)),
            last_notify_ms: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn notify_folder(&self, path: &Path) {
        let count = self.files_scanned.load(Ordering::Relaxed);
        let total_bytes = self.bytes_scanned.load(Ordering::Relaxed);
        let _ = self.sender.send(ScanEvent::Progress {
            scanned_count: count,
            current_path: path.to_string_lossy().to_string(),
            current_bytes: total_bytes,
        });
    }

    pub fn record_file(&self, path: &Path, bytes: u64) {
        let count = self.files_scanned.fetch_add(1, Ordering::Relaxed) + 1;
        let total_bytes = self.bytes_scanned.fetch_add(bytes, Ordering::Relaxed) + bytes;

        let now_ms = self.start_time.elapsed().as_millis() as u64;
        let last_ms = self.last_notify_ms.load(Ordering::Relaxed);

        // Throttle fluido: emitir actualización cada 100 archivos o cada 50ms
        if count % 100 == 0 || now_ms.saturating_sub(last_ms) >= 50 {
            if self
                .last_notify_ms
                .compare_exchange(last_ms, now_ms, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                let _ = self.sender.send(ScanEvent::Progress {
                    scanned_count: count,
                    current_path: path.to_string_lossy().to_string(),
                    current_bytes: total_bytes,
                });
            }
        }
    }
}

pub fn start_scan_thread(
    root: PathBuf,
    limit_mb: f64,
    max_depth: usize,
    cache: Arc<Mutex<HashMap<PathBuf, ScanResult>>>,
    cancel_token: Arc<AtomicBool>,
    sender: Sender<ScanEvent>,
) {
    thread::spawn(move || {
        let (disk_free, disk_total) = get_disk_info(&root);
        let reporter = ScanReporter::new(sender.clone());
        reporter.notify_folder(&root);

        let mut total_root_bytes: u64 = 0;
        let mut items = Vec::new();

        let entries = match fs::read_dir(&root) {
            Ok(e) => e,
            Err(err) => {
                let _ = sender.send(ScanEvent::Error(format!(
                    "No se pudo acceder a '{}': {}",
                    root.display(),
                    err
                )));
                return;
            }
        };

        let mut sub_dirs = Vec::new();

        // 1. Escanear archivos directos de la raíz y recopilar carpetas
        for entry_res in entries {
            if cancel_token.load(Ordering::Relaxed) {
                let _ = sender.send(ScanEvent::Cancelled);
                return;
            }

            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if should_skip_path(&path) {
                continue;
            }

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            // Omitir enlaces simbólicos para evitar bucles y conteo doble
            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_file() {
                if let Ok(meta) = entry.metadata() {
                    let size = meta.len();
                    total_root_bytes += size;
                    reporter.record_file(&path, size);

                    let size_mb = (size as f64) / (1024.0 * 1024.0);
                    let is_excessive = size_mb > limit_mb;

                    let file_item = ScanItem {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: path.clone(),
                        item_type: ItemType::File,
                        size_bytes: size,
                        size_mb,
                        is_excessive,
                    };

                    let _ = sender.send(ScanEvent::ItemDiscovered(file_item.clone()));
                    items.push(file_item);
                }
            } else if file_type.is_dir() {
                sub_dirs.push((entry.file_name().to_string_lossy().to_string(), path));
            }
        }

        // 2. Escanear subdirectorios directos de la raíz en paralelo con Rayon aprovechando todos los núcleos CPU
        let dir_items: Vec<ScanItem> = sub_dirs
            .into_par_iter()
            .filter_map(|(dir_name, dir_path)| {
                if cancel_token.load(Ordering::Relaxed) {
                    return None;
                }

                // Notificar avance de esta carpeta a la interfaz
                reporter.notify_folder(&dir_path);

                let dir_size = if max_depth > 1 {
                    scan_dir_recursive(
                        &dir_path,
                        1,
                        max_depth,
                        limit_mb,
                        disk_free,
                        disk_total,
                        &cache,
                        &cancel_token,
                        &reporter,
                    )
                } else {
                    calculate_dir_size_fast(&dir_path, &cancel_token, &reporter)
                };

                if cancel_token.load(Ordering::Relaxed) {
                    return None;
                }

                let size_mb = (dir_size as f64) / (1024.0 * 1024.0);
                let is_excessive = size_mb > limit_mb;

                let dir_item = ScanItem {
                    name: dir_name,
                    path: dir_path,
                    item_type: ItemType::Folder,
                    size_bytes: dir_size,
                    size_mb,
                    is_excessive,
                };

                let _ = sender.send(ScanEvent::ItemDiscovered(dir_item.clone()));
                Some(dir_item)
            })
            .collect();

        if cancel_token.load(Ordering::Relaxed) {
            let _ = sender.send(ScanEvent::Cancelled);
            return;
        }

        for item in dir_items {
            total_root_bytes += item.size_bytes;
            items.push(item);
        }

        // Ordenar items: Carpetas primero, luego por orden alfabético
        sort_items_default(&mut items);

        let total_size_mb = (total_root_bytes as f64) / (1024.0 * 1024.0);

        let result = ScanResult {
            path: root.clone(),
            items,
            total_size_bytes: total_root_bytes,
            total_size_mb,
            disk_free_bytes: disk_free,
            disk_total_bytes: disk_total,
        };

        // Guardar la raíz en caché
        if let Ok(mut c) = cache.lock() {
            c.insert(root, result.clone());
        }

        let _ = sender.send(ScanEvent::Completed(Box::new(result)));
    });
}

fn scan_dir_recursive(
    dir_path: &Path,
    current_depth: usize,
    max_depth: usize,
    limit_mb: f64,
    disk_free: u64,
    disk_total: u64,
    cache: &Arc<Mutex<HashMap<PathBuf, ScanResult>>>,
    cancel_token: &Arc<AtomicBool>,
    reporter: &ScanReporter,
) -> u64 {
    if cancel_token.load(Ordering::Relaxed) {
        return 0;
    }

    let entries = match fs::read_dir(dir_path) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut direct_file_bytes: u64 = 0;
    let mut file_items = Vec::new();
    let mut sub_dirs = Vec::new();

    for entry_res in entries {
        if cancel_token.load(Ordering::Relaxed) {
            return direct_file_bytes;
        }

        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if should_skip_path(&path) {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_file() {
            if let Ok(meta) = entry.metadata() {
                let f_size = meta.len();
                direct_file_bytes += f_size;
                reporter.record_file(&path, f_size);

                let size_mb = (f_size as f64) / (1024.0 * 1024.0);

                file_items.push(ScanItem {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path,
                    item_type: ItemType::File,
                    size_bytes: f_size,
                    size_mb,
                    is_excessive: size_mb > limit_mb,
                });
            }
        } else if file_type.is_dir() {
            sub_dirs.push((entry.file_name().to_string_lossy().to_string(), path));
        }
    }

    // Paralelizar con Rayon en niveles superiores para balancear carga en árboles profundos
    let folder_items: Vec<ScanItem> = if current_depth < 3 && sub_dirs.len() > 1 {
        sub_dirs
            .into_par_iter()
            .filter_map(|(dir_name, sub_dir_path)| {
                if cancel_token.load(Ordering::Relaxed) {
                    return None;
                }

                let sub_size = if current_depth < max_depth {
                    scan_dir_recursive(
                        &sub_dir_path,
                        current_depth + 1,
                        max_depth,
                        limit_mb,
                        disk_free,
                        disk_total,
                        cache,
                        cancel_token,
                        reporter,
                    )
                } else {
                    calculate_dir_size_fast(&sub_dir_path, cancel_token, reporter)
                };

                let size_mb = (sub_size as f64) / (1024.0 * 1024.0);

                Some(ScanItem {
                    name: dir_name,
                    path: sub_dir_path,
                    item_type: ItemType::Folder,
                    size_bytes: sub_size,
                    size_mb,
                    is_excessive: size_mb > limit_mb,
                })
            })
            .collect()
    } else {
        let mut f_items = Vec::with_capacity(sub_dirs.len());
        for (dir_name, sub_dir_path) in sub_dirs {
            if cancel_token.load(Ordering::Relaxed) {
                break;
            }

            let sub_size = if current_depth < max_depth {
                scan_dir_recursive(
                    &sub_dir_path,
                    current_depth + 1,
                    max_depth,
                    limit_mb,
                    disk_free,
                    disk_total,
                    cache,
                    cancel_token,
                    reporter,
                )
            } else {
                calculate_dir_size_fast(&sub_dir_path, cancel_token, reporter)
            };

            let size_mb = (sub_size as f64) / (1024.0 * 1024.0);

            f_items.push(ScanItem {
                name: dir_name,
                path: sub_dir_path,
                item_type: ItemType::Folder,
                size_bytes: sub_size,
                size_mb,
                is_excessive: size_mb > limit_mb,
            });
        }
        f_items
    };

    let total_dir_size: u64 =
        direct_file_bytes + folder_items.iter().map(|i| i.size_bytes).sum::<u64>();

    let mut all_items = file_items;
    all_items.extend(folder_items);
    sort_items_default(&mut all_items);

    // Guardar en caché este nivel también
    if !cancel_token.load(Ordering::Relaxed) {
        if let Ok(mut c) = cache.lock() {
            c.insert(
                dir_path.to_path_buf(),
                ScanResult {
                    path: dir_path.to_path_buf(),
                    items: all_items,
                    total_size_bytes: total_dir_size,
                    total_size_mb: (total_dir_size as f64) / (1024.0 * 1024.0),
                    disk_free_bytes: disk_free,
                    disk_total_bytes: disk_total,
                },
            );
        }
    }

    total_dir_size
}

pub fn calculate_dir_size_fast(
    dir_path: &Path,
    cancel_token: &Arc<AtomicBool>,
    reporter: &ScanReporter,
) -> u64 {
    if cancel_token.load(Ordering::Relaxed) {
        return 0;
    }

    let mut size: u64 = 0;
    let entries = match fs::read_dir(dir_path) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    for entry_res in entries {
        if cancel_token.load(Ordering::Relaxed) {
            return size;
        }

        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if should_skip_path(&path) {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_file() {
            if let Ok(meta) = entry.metadata() {
                let f_size = meta.len();
                size += f_size;
                reporter.record_file(&path, f_size);
            }
        } else if file_type.is_dir() {
            size += calculate_dir_size_fast(&path, cancel_token, reporter);
        }
    }

    size
}

