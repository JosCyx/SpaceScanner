use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use eframe::egui::{self, Color32, RichText, Rounding, Sense, Vec2};
use egui_extras::{Column, TableBuilder};

use crate::models::{
    sort_items_default, ItemType, ScanEvent, ScanItem, ScanResult, SortColumn, ViewMode,
};
use crate::scanner::{get_disk_info, parse_size_limit, start_scan_thread};

pub struct SpaceScannerApp {
    // Entradas
    path_input: String,
    size_input: String,
    deep_scan: bool,
    levels: usize,

    // Estado del escaneo
    is_scanning: bool,
    status_text: String,
    show_stop_confirm: bool,
    show_error_dialog: Option<String>,
    last_error: Option<String>,
    from_cache: bool,

    // Historial y Caché
    navigation_history: Vec<PathBuf>,
    forward_history: Vec<PathBuf>,
    current_path: PathBuf,
    scan_cache: Arc<Mutex<HashMap<PathBuf, ScanResult>>>,

    // Datos actuales
    current_result: Option<ScanResult>,
    stream_items: Vec<ScanItem>,
    view_mode: ViewMode,

    // Ordenamiento por columnas (None = orden por defecto: carpetas primero, luego archivos, A-Z)
    sort_state: Option<(SortColumn, bool)>,

    // Métricas de disco
    disk_free: u64,
    disk_total: u64,

    // Canales y cancelación
    cancel_token: Arc<AtomicBool>,
    sender: Sender<ScanEvent>,
    receiver: Receiver<ScanEvent>,
}

impl SpaceScannerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = channel();

        // Carpeta inicial: home del usuario
        let initial_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let initial_path = PathBuf::from(&initial_dir);

        let (free, total) = get_disk_info(&initial_path);

        Self {
            path_input: initial_dir,
            size_input: "5000 MB".to_string(),
            deep_scan: true,
            levels: 7,

            is_scanning: false,
            status_text: "Listo para escanear".to_string(),
            show_stop_confirm: false,
            show_error_dialog: None,
            last_error: None,
            from_cache: false,

            navigation_history: Vec::new(),
            forward_history: Vec::new(),
            current_path: initial_path,
            scan_cache: Arc::new(Mutex::new(HashMap::new())),

            current_result: None,
            stream_items: Vec::new(),
            view_mode: ViewMode::Detailed,

            sort_state: None, // Por defecto: Carpetas y Archivos en orden alfabético

            disk_free: free,
            disk_total: total,

            cancel_token: Arc::new(AtomicBool::new(false)),
            sender,
            receiver,
        }
    }

    fn aplicar_ordenamiento(&mut self) {
        if let Some((col, asc)) = self.sort_state {
            if let Some(res) = &mut self.current_result {
                col.sort_items(&mut res.items, asc);
            }
            col.sort_items(&mut self.stream_items, asc);
        } else {
            if let Some(res) = &mut self.current_result {
                sort_items_default(&mut res.items);
            }
            sort_items_default(&mut self.stream_items);
        }
    }

    fn iniciar_escaneo(&mut self, push_historial: bool) {
        let trimmed = self.path_input.trim();
        let path = PathBuf::from(trimmed);

        if trimmed.is_empty() {
            self.show_error_dialog = Some("Por favor ingrese una ruta válida.".to_string());
            return;
        }

        if !path.exists() {
            self.show_error_dialog = Some("La ruta especificada no existe.".to_string());
            return;
        }

        if path.is_file() {
            self.show_error_dialog = Some(
                "No se puede escanear un archivo. Por favor seleccione una carpeta válida."
                    .to_string(),
            );
            return;
        }

        if push_historial {
            if self.current_path.exists() && self.current_path != path {
                self.navigation_history.push(self.current_path.clone());
            }
            self.forward_history.clear();
        }

        self.current_path = path.clone();
        self.from_cache = false;
        self.last_error = None;

        let limit_mb = parse_size_limit(&self.size_input);

        // Verificar si está en caché
        if let Ok(cache) = self.scan_cache.lock() {
            if let Some(cached) = cache.get(&path) {
                let mut cached_result = cached.clone();
                // Actualizar estado de items según el límite actual
                for item in &mut cached_result.items {
                    item.is_excessive = item.size_mb > limit_mb;
                }
                if let Some((col, asc)) = self.sort_state {
                    col.sort_items(&mut cached_result.items, asc);
                } else {
                    sort_items_default(&mut cached_result.items);
                }
                self.disk_free = cached_result.disk_free_bytes;
                self.disk_total = cached_result.disk_total_bytes;
                self.current_result = Some(cached_result);
                self.from_cache = true;
                self.status_text = format!(
                    "Total ruta: {:.2} MB (desde memoria)",
                    self.current_result.as_ref().unwrap().total_size_mb
                );
                return;
            }
        }

        // Si no está en caché, comenzar escaneo en background
        self.is_scanning = true;
        self.status_text = "Calculando...".to_string();
        self.stream_items.clear();
        self.current_result = None;

        let (free, total) = get_disk_info(&path);
        self.disk_free = free;
        self.disk_total = total;

        let max_depth = if self.deep_scan { self.levels } else { 1 };
        // Cancelar escaneo anterior si estuviese activo
        self.cancel_token.store(true, Ordering::Relaxed);
        // Crear un nuevo token atómico para el nuevo escaneo
        self.cancel_token = Arc::new(AtomicBool::new(false));

        start_scan_thread(
            path,
            limit_mb,
            max_depth,
            Arc::clone(&self.scan_cache),
            Arc::clone(&self.cancel_token),
            self.sender.clone(),
        );
    }

    fn retroceder(&mut self) {
        if let Some(prev) = self.navigation_history.pop() {
            if self.current_path.exists() {
                self.forward_history.push(self.current_path.clone());
            }
            self.path_input = prev.to_string_lossy().to_string();
            self.iniciar_escaneo(false);
        }
    }

    fn avanzar(&mut self) {
        if let Some(next) = self.forward_history.pop() {
            if self.current_path.exists() {
                self.navigation_history.push(self.current_path.clone());
            }
            self.path_input = next.to_string_lossy().to_string();
            self.iniciar_escaneo(false);
        }
    }

    fn procesar_eventos(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                ScanEvent::Progress {
                    scanned_count,
                    current_path,
                    current_bytes,
                } => {
                    let mb = (current_bytes as f64) / (1024.0 * 1024.0);
                    let display_path = if current_path.chars().count() > 45 {
                        let end_chars: String = current_path
                            .chars()
                            .rev()
                            .take(42)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect();
                        format!("...{}", end_chars)
                    } else {
                        current_path
                    };

                    self.status_text = format!(
                        "Escaneando: {} | {} elementos | {:.1} MB",
                        display_path, scanned_count, mb
                    );
                }
                ScanEvent::ItemDiscovered(item) => {
                    self.stream_items.push(item);
                    if let Some((col, asc)) = self.sort_state {
                        col.sort_items(&mut self.stream_items, asc);
                    } else {
                        sort_items_default(&mut self.stream_items);
                    }
                }
                ScanEvent::Completed(mut res) => {
                    self.is_scanning = false;
                    if let Some((col, asc)) = self.sort_state {
                        col.sort_items(&mut res.items, asc);
                    } else {
                        sort_items_default(&mut res.items);
                    }
                    self.status_text = format!(
                        "Total ruta: {:.2} MB ({} elementos analizados)",
                        res.total_size_mb,
                        res.items.len()
                    );
                    self.disk_free = res.disk_free_bytes;
                    self.disk_total = res.disk_total_bytes;
                    self.current_result = Some(*res);
                    self.last_error = None;
                }
                ScanEvent::Cancelled => {
                    self.is_scanning = false;
                    self.status_text = "Escaneo detenido por el usuario".to_string();
                }
                ScanEvent::Error(err) => {
                    self.is_scanning = false;
                    self.status_text = format!("Error: {}", err);
                    self.last_error = Some(err);
                }
            }
        }
    }
}

impl eframe::App for SpaceScannerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.procesar_eventos();

        if self.is_scanning {
            ctx.request_repaint(); // Redibujar fluido durante el escaneo
        }

        // Atajos de navegación: botones laterales del ratón (Atrás / Adelante) y teclado
        let mouse_back = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra1));
        let mouse_fwd = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra2));
        let alt_left = !ctx.wants_keyboard_input()
            && ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft));
        let alt_right = !ctx.wants_keyboard_input()
            && ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight));

        if (mouse_back || alt_left) && !self.is_scanning && !self.navigation_history.is_empty() {
            self.retroceder();
        } else if (mouse_fwd || alt_right) && !self.is_scanning && !self.forward_history.is_empty() {
            self.avanzar();
        }

        // ==================== PANEL SUPERIOR ====================
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);

            // FILA 1: Barra de Dirección y Acciones Principales
            ui.horizontal(|ui| {
                ui.label(RichText::new("Ruta:").strong().size(13.5));

                let available = ui.available_width() - 480.0;
                let text_edit = ui.add_sized(
                    [available.max(160.0), 28.0],
                    egui::TextEdit::singleline(&mut self.path_input)
                        .hint_text("Ruta de carpeta a escanear...")
                        .interactive(!self.is_scanning),
                );

                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.iniciar_escaneo(true);
                }

                // Botón Explorar Carpeta
                let browse_btn = egui::Button::new(RichText::new("📁 Examinar").strong().size(12.5))
                    .min_size(Vec2::new(0.0, 28.0))
                    .rounding(Rounding::same(4.0));

                if ui.add_enabled(!self.is_scanning, browse_btn).clicked() {
                    if let Some(folder) = rfd::FileDialog::new()
                        .set_directory(&self.path_input)
                        .pick_folder()
                    {
                        self.path_input = folder.to_string_lossy().to_string();
                    }
                }

                // Botón Escanear
                let scan_btn = egui::Button::new(
                    RichText::new("🔍 Escanear")
                        .color(Color32::WHITE)
                        .strong()
                        .size(12.5),
                )
                .fill(Color32::from_rgb(37, 99, 235))
                .min_size(Vec2::new(0.0, 28.0))
                .rounding(Rounding::same(4.0));

                if ui.add_enabled(!self.is_scanning, scan_btn).clicked() {
                    self.iniciar_escaneo(true);
                }

                // Botón Parar
                let stop_btn = egui::Button::new(
                    RichText::new("⏹ Parar")
                        .strong()
                        .size(12.5)
                        .color(if self.is_scanning {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(180, 185, 195)
                        }),
                )
                .fill(if self.is_scanning {
                    Color32::from_rgb(220, 38, 38)
                } else {
                    Color32::from_rgb(50, 54, 65)
                })
                .min_size(Vec2::new(0.0, 28.0))
                .rounding(Rounding::same(4.0));

                if ui.add_enabled(self.is_scanning, stop_btn).clicked() {
                    self.show_stop_confirm = true;
                }

                // Botón Atrás
                let back_enabled = !self.navigation_history.is_empty() && !self.is_scanning;
                let back_text = if self.navigation_history.is_empty() {
                    "↩ Atrás".to_string()
                } else {
                    format!("↩ Atrás ({})", self.navigation_history.len())
                };

                let back_btn = egui::Button::new(RichText::new(back_text).strong().size(12.5))
                    .min_size(Vec2::new(0.0, 28.0))
                    .rounding(Rounding::same(4.0));

                if ui.add_enabled(back_enabled, back_btn).clicked() {
                    self.retroceder();
                }

                // Botón Adelante
                let fwd_enabled = !self.forward_history.is_empty() && !self.is_scanning;
                let fwd_text = if self.forward_history.is_empty() {
                    "↪ Adelante".to_string()
                } else {
                    format!("↪ Adelante ({})", self.forward_history.len())
                };

                let fwd_btn = egui::Button::new(RichText::new(fwd_text).strong().size(12.5))
                    .min_size(Vec2::new(0.0, 28.0))
                    .rounding(Rounding::same(4.0));

                if ui.add_enabled(fwd_enabled, fwd_btn).clicked() {
                    self.avanzar();
                }
            });

            ui.add_space(6.0);

            // FILA 2: Opciones de Escaneo y Vistas
            ui.horizontal(|ui| {
                ui.label("Límite:");
                ui.add_sized(
                    [85.0, 24.0],
                    egui::TextEdit::singleline(&mut self.size_input)
                        .hint_text("5000 MB")
                        .interactive(!self.is_scanning),
                );

                ui.separator();

                ui.checkbox(&mut self.deep_scan, "Escaneo profundo");

                ui.add_enabled_ui(self.deep_scan, |ui| {
                    ui.label("Niveles:");
                    ui.add(egui::DragValue::new(&mut self.levels).range(1..=20));
                });

                ui.separator();

                // Selector de Vistas
                ui.selectable_value(&mut self.view_mode, ViewMode::Detailed, "📋 Detallada");
                ui.selectable_value(&mut self.view_mode, ViewMode::Visual, "📊 Gráfica");

                ui.separator();
                let sort_label = match self.sort_state {
                    None => "Orden: Carpetas y Archivos (A-Z)".to_string(),
                    Some((col, asc)) => {
                        let col_name = match col {
                            SortColumn::Name => "Nombre",
                            SortColumn::Path => "Ruta",
                            SortColumn::Type => "Tipo",
                            SortColumn::Size => "Tamaño",
                            SortColumn::Status => "Estado",
                        };
                        let dir_str = if asc { "Ascendente" } else { "Descendente" };
                        format!("Orden: {} ({})", col_name, dir_str)
                    }
                };
                ui.label(
                    RichText::new(sort_label)
                        .small()
                        .color(Color32::from_rgb(156, 163, 175)),
                );

                if self.from_cache {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("⚡ Datos cargados desde memoria caché")
                                .color(Color32::from_rgb(245, 158, 11))
                                .small()
                                .italics(),
                        );
                    });
                }
            });

            ui.add_space(6.0);
        });

        // ==================== PANEL INFERIOR (Status Bar) ====================
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // Indicador visual con Spinner cuando está escaneando
                if self.is_scanning {
                    ui.add(egui::Spinner::new().size(15.0));
                    ui.colored_label(
                        Color32::from_rgb(96, 165, 250),
                        RichText::new(&self.status_text).strong(),
                    );
                } else if self.status_text.contains("Total ruta") {
                    ui.colored_label(
                        Color32::from_rgb(34, 197, 94),
                        RichText::new(format!("✅ {}", self.status_text)).strong(),
                    );
                } else if self.status_text.contains("detenido") || self.status_text.contains("cancelado") {
                    ui.colored_label(
                        Color32::from_rgb(245, 158, 11),
                        RichText::new(format!("⏹ {}", self.status_text)).strong(),
                    );
                } else if self.status_text.starts_with("Error") {
                    ui.colored_label(
                        Color32::from_rgb(239, 68, 68),
                        RichText::new(format!("❌ {}", self.status_text)).strong(),
                    );
                } else {
                    ui.label(RichText::new(&self.status_text).strong());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_gb = (self.disk_total as f64) / (1024.0 * 1024.0 * 1024.0);
                    let free_gb = (self.disk_free as f64) / (1024.0 * 1024.0 * 1024.0);
                    let used_ratio = if self.disk_total > 0 {
                        ((self.disk_total - self.disk_free) as f32) / (self.disk_total as f32)
                    } else {
                        0.0
                    };

                    ui.label(format!(
                        "Disco: {:.1} GB libres de {:.1} GB",
                        free_gb, total_gb
                    ));

                    // Mini barra de uso de disco elegante
                    let bar_size = Vec2::new(100.0, 10.0);
                    let (rect, _) = ui.allocate_exact_size(bar_size, Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        Rounding::same(2.0),
                        Color32::from_rgb(60, 64, 72),
                    );
                    let mut used_rect = rect;
                    used_rect.set_width(rect.width() * used_ratio.clamp(0.0, 1.0));
                    ui.painter().rect_filled(
                        used_rect,
                        Rounding::same(2.0),
                        Color32::from_rgb(59, 130, 246),
                    );
                });
            });
            ui.add_space(4.0);
        });

        // ==================== PANEL CENTRAL RESPONSIVO ====================
        let mut navigate_to: Option<PathBuf> = None;
        let mut open_resource: Option<PathBuf> = None;
        let mut dismiss_error = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            // Banner de error si ocurrió algún fallo
            if let Some(err) = &self.last_error {
                let err_frame = egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(50, 20, 20))
                    .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(239, 68, 68)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(8.0);

                err_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            Color32::from_rgb(239, 68, 68),
                            RichText::new("⚠️ Error durante el escaneo:").strong(),
                        );
                        ui.label(RichText::new(err).color(Color32::from_rgb(254, 202, 202)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✕ Cerrar").clicked() {
                                dismiss_error = true;
                            }
                        });
                    });
                });
                ui.add_space(6.0);
            }

            let items: &[ScanItem] = if let Some(res) = &self.current_result {
                &res.items
            } else {
                &self.stream_items
            };

            let mut change_sort: Option<SortColumn> = None;

            match self.view_mode {
                ViewMode::Detailed => {
                    let (nav, open, new_sort) = Self::render_detailed_table(
                        ui,
                        items,
                        self.sort_state,
                    );
                    navigate_to = nav;
                    open_resource = open;
                    change_sort = new_sort;
                }
                ViewMode::Visual => {
                    let (nav, open) = Self::render_visual_chart(ui, items);
                    navigate_to = nav;
                    open_resource = open;
                }
            }

            if let Some(col) = change_sort {
                match self.sort_state {
                    None => {
                        // 1er clic: activar la columna seleccionada
                        let initial_asc = col != SortColumn::Size && col != SortColumn::Status;
                        self.sort_state = Some((col, initial_asc));
                    }
                    Some((curr_col, asc)) => {
                        if curr_col == col {
                            let initial_asc = col != SortColumn::Size && col != SortColumn::Status;
                            if asc == initial_asc {
                                // 2do clic: invertir dirección
                                self.sort_state = Some((col, !initial_asc));
                            } else {
                                // 3er clic: quitar el ordenamiento (vuelve al orden por defecto)
                                self.sort_state = None;
                            }
                        } else {
                            // Clic en otra columna
                            let initial_asc = col != SortColumn::Size && col != SortColumn::Status;
                            self.sort_state = Some((col, initial_asc));
                        }
                    }
                }
                self.aplicar_ordenamiento();
            }
        });

        if dismiss_error {
            self.last_error = None;
        }

        if let Some(folder) = navigate_to {
            self.path_input = folder.to_string_lossy().to_string();
            self.iniciar_escaneo(true);
        }

        if let Some(path) = open_resource {
            let _ = open::that(path);
        }

        // ==================== MODAL DE CONFIRMACIÓN ====================
        if self.show_stop_confirm {
            egui::Window::new("Confirmar cancelación")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("¿Está seguro que desea cancelar el escaneo actual?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Sí, cancelar").clicked() {
                            self.cancel_token.store(true, Ordering::Relaxed);
                            self.show_stop_confirm = false;
                        }
                        if ui.button("Continuar escaneando").clicked() {
                            self.show_stop_confirm = false;
                        }
                    });
                });
        }

        // ==================== DIÁLOGO DE ERROR ====================
        if let Some(err) = self.show_error_dialog.clone() {
            egui::Window::new("Aviso")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new(&err).color(Color32::from_rgb(239, 68, 68)));
                    ui.add_space(10.0);
                    if ui.button("Aceptar").clicked() {
                        self.show_error_dialog = None;
                    }
                });
        }
    }
}

impl SpaceScannerApp {
    fn render_detailed_table(
        ui: &mut egui::Ui,
        items: &[ScanItem],
        sort_state: Option<(SortColumn, bool)>,
    ) -> (Option<PathBuf>, Option<PathBuf>, Option<SortColumn>) {
        let mut clicked_folder: Option<PathBuf> = None;
        let mut action_open: Option<PathBuf> = None;
        let mut action_scan: Option<PathBuf> = None;
        let mut clicked_sort: Option<SortColumn> = None;

        TableBuilder::new(ui)
            .striped(true)
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(220.0).resizable(true).clip(true)) // Nombre
            .column(Column::remainder().resizable(true).clip(true))    // Ruta
            .column(Column::exact(100.0))                             // Tipo
            .column(Column::exact(135.0))                             // Tamaño
            .column(Column::exact(115.0))                             // Estado
            .header(28.0, |mut header| {
                header.col(|ui| {
                    if Self::render_header_btn(ui, "Nombre", SortColumn::Name, sort_state) {
                        clicked_sort = Some(SortColumn::Name);
                    }
                });
                header.col(|ui| {
                    if Self::render_header_btn(ui, "Ruta", SortColumn::Path, sort_state) {
                        clicked_sort = Some(SortColumn::Path);
                    }
                });
                header.col(|ui| {
                    if Self::render_header_btn(ui, "Tipo", SortColumn::Type, sort_state) {
                        clicked_sort = Some(SortColumn::Type);
                    }
                });
                header.col(|ui| {
                    if Self::render_header_btn(ui, "Tamaño (MB)", SortColumn::Size, sort_state) {
                        clicked_sort = Some(SortColumn::Size);
                    }
                });
                header.col(|ui| {
                    if Self::render_header_btn(ui, "Estado", SortColumn::Status, sort_state) {
                        clicked_sort = Some(SortColumn::Status);
                    }
                });
            })
            .body(|body| {
                body.rows(26.0, items.len(), |mut row| {
                    let item = &items[row.index()];

                    // Columna 1: Nombre con icono
                    row.col(|ui| {
                        let icon = if item.item_type == ItemType::Folder { "📁 " } else { "📄 " };
                        ui.label(RichText::new(format!("{}{}", icon, item.name)).strong());
                    });

                    // Columna 2: Ruta completa
                    row.col(|ui| {
                        let path_str = item.path.to_string_lossy();
                        ui.label(RichText::new(path_str).color(Color32::from_rgb(156, 163, 175)).small());
                    });

                    // Columna 3: Tipo
                    row.col(|ui| {
                        if item.item_type == ItemType::Folder {
                            ui.colored_label(Color32::from_rgb(245, 158, 11), "Carpeta");
                        } else {
                            ui.colored_label(Color32::from_rgb(148, 163, 184), "Archivo");
                        }
                    });

                    // Columna 4: Tamaño en MB
                    row.col(|ui| {
                        ui.label(format!("{:.2} MB", item.size_mb));
                    });

                    // Columna 5: Estado
                    row.col(|ui| {
                        if item.is_excessive {
                            ui.colored_label(
                                Color32::from_rgb(239, 68, 68),
                                RichText::new("⚠️ Excesivo").strong(),
                            );
                        } else {
                            ui.colored_label(
                                Color32::from_rgb(34, 197, 94),
                                RichText::new("✅ Normal"),
                            );
                        }
                    });

                    // Interacción en CUALQUIER lugar de la fila completa
                    let row_resp = row.response();

                    // Doble clic en cualquier celda o espacio de la fila
                    if row_resp.double_clicked() && item.item_type == ItemType::Folder {
                        clicked_folder = Some(item.path.clone());
                    }

                    // Menú contextual clic derecho en cualquier celda o espacio de la fila
                    row_resp.context_menu(|ui| {
                        if ui.button("📂 Abrir recurso").clicked() {
                            action_open = Some(item.path.clone());
                            ui.close_menu();
                        }
                        if item.item_type == ItemType::Folder {
                            if ui.button("🔍 Escanear esta carpeta").clicked() {
                                action_scan = Some(item.path.clone());
                                ui.close_menu();
                            }
                        }
                    });
                });
            });

        (clicked_folder.or(action_scan), action_open, clicked_sort)
    }

    fn render_header_btn(
        ui: &mut egui::Ui,
        title: &str,
        col: SortColumn,
        sort_state: Option<(SortColumn, bool)>,
    ) -> bool {
        let (is_active, ascending) = match sort_state {
            Some((c, asc)) if c == col => (true, asc),
            _ => (false, false),
        };

        let w = ui.available_width();
        let h = 26.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

        // Estilo de fondo y borde
        let (bg_color, border_color) = if is_active {
            (Color32::from_rgb(30, 42, 60), Color32::from_rgb(59, 130, 246))
        } else if response.hovered() {
            (Color32::from_rgb(36, 42, 54), Color32::from_rgb(70, 78, 96))
        } else {
            (Color32::from_rgb(24, 27, 34), Color32::from_rgb(45, 50, 62))
        };

        ui.painter().rect(
            rect,
            Rounding::same(3.0),
            bg_color,
            egui::Stroke::new(1.0_f32, border_color),
        );

        // Texto del título a la izquierda (padding de 8px)
        let title_color = if is_active {
            Color32::from_rgb(147, 197, 253)
        } else if response.hovered() {
            Color32::WHITE
        } else {
            Color32::from_rgb(209, 213, 219)
        };

        let title_pos = egui::pos2(rect.left() + 8.0, rect.center().y);
        ui.painter().text(
            title_pos,
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(12.5),
            title_color,
        );

        // Chevron vectorial minimalista e independiente de fuentes (renderizado vectorial SVG nativo)
        if is_active {
            let cx = rect.right() - 14.0;
            let cy = rect.center().y;
            let stroke = egui::Stroke::new(1.8_f32, Color32::from_rgb(96, 165, 250));

            if ascending {
                // Chevron apuntando hacia arriba (Ascendente)
                let p_left = egui::pos2(cx - 3.5, cy + 2.0);
                let p_tip = egui::pos2(cx, cy - 2.5);
                let p_right = egui::pos2(cx + 3.5, cy + 2.0);
                ui.painter().line_segment([p_left, p_tip], stroke);
                ui.painter().line_segment([p_tip, p_right], stroke);
            } else {
                // Chevron apuntando hacia abajo (Descendente)
                let p_left = egui::pos2(cx - 3.5, cy - 2.0);
                let p_tip = egui::pos2(cx, cy + 2.5);
                let p_right = egui::pos2(cx + 3.5, cy - 2.0);
                ui.painter().line_segment([p_left, p_tip], stroke);
                ui.painter().line_segment([p_tip, p_right], stroke);
            }
        }

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        response.clicked()
    }

    fn render_visual_chart(
        ui: &mut egui::Ui,
        items: &[ScanItem],
    ) -> (Option<PathBuf>, Option<PathBuf>) {
        let mut navigate_to: Option<PathBuf> = None;
        let mut action_open: Option<PathBuf> = None;

        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Distribución visual del espacio en esta carpeta")
                    .strong()
                    .size(15.0),
            );
            ui.label(
                RichText::new("Haz clic en cualquier barra de carpeta para explorarla directamente.")
                    .small()
                    .color(Color32::from_rgb(156, 163, 175)),
            );
            ui.add_space(10.0);

            if items.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No hay elementos para mostrar en la gráfica.");
                });
            } else {
                let total_bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
                if total_bytes == 0 {
                    ui.label("La carpeta está vacía o contiene 0 bytes.");
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            let full_w = ui.available_width();

                            for item in items.iter().take(40) {
                                let percentage = (item.size_bytes as f64 / total_bytes as f64) * 100.0;

                                let frame = egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(26, 29, 36))
                                    .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(45, 50, 62)))
                                    .rounding(Rounding::same(5.0))
                                    .inner_margin(8.0);

                                frame.show(ui, |ui| {
                                    ui.set_width(full_w);

                            ui.horizontal(|ui| {
                                let icon = if item.item_type == ItemType::Folder { "📁 " } else { "📄 " };
                                let label = format!("{}{}", icon, item.name);

                                ui.add_sized(
                                    [220.0, 20.0],
                                    egui::Label::new(RichText::new(label).strong()).truncate(),
                                );

                                ui.label(format!("{:.2} MB ({:.1}%)", item.size_mb, percentage));

                                if item.is_excessive {
                                    ui.colored_label(
                                        Color32::from_rgb(239, 68, 68),
                                        RichText::new("⚠️ Excesivo").small().strong(),
                                    );
                                }

                                if item.item_type == ItemType::Folder {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("🔍 Explorar").clicked() {
                                            navigate_to = Some(item.path.clone());
                                        }
                                    });
                                }
                            });

                            ui.add_space(6.0);

                            // Barra que abarca TODO el ancho del card
                            let bar_ratio = (percentage as f32 / 100.0).clamp(0.005, 1.0);
                            let bar_color = if item.is_excessive {
                                Color32::from_rgb(239, 68, 68) // Rojo alerta
                            } else if item.item_type == ItemType::Folder {
                                Color32::from_rgb(245, 158, 11) // Ámbar carpeta
                            } else {
                                Color32::from_rgb(59, 130, 246) // Azul archivo
                            };

                            let track_w = ui.available_width();
                            let (track_rect, resp) = ui.allocate_exact_size(
                                Vec2::new(track_w, 16.0),
                                Sense::click(),
                            );

                            // Fondo del track de la barra (100% del ancho)
                            ui.painter().rect_filled(
                                track_rect,
                                Rounding::same(3.0),
                                Color32::from_rgb(38, 42, 53),
                            );

                            // Barra llena proporcional
                            let mut filled_rect = track_rect;
                            filled_rect.set_width((track_rect.width() * bar_ratio).max(4.0));
                            ui.painter().rect_filled(
                                filled_rect,
                                Rounding::same(3.0),
                                bar_color,
                            );

                            if (resp.clicked() || resp.double_clicked()) && item.item_type == ItemType::Folder {
                                navigate_to = Some(item.path.clone());
                            }

                            resp.context_menu(|ui| {
                                if ui.button("📂 Abrir recurso").clicked() {
                                    action_open = Some(item.path.clone());
                                    ui.close_menu();
                                }
                                if item.item_type == ItemType::Folder {
                                    if ui.button("🔍 Escanear esta carpeta").clicked() {
                                        navigate_to = Some(item.path.clone());
                                        ui.close_menu();
                                    }
                                }
                            });
                        });

                        ui.add_space(6.0);
                    }
                });
                }
            }
        });

        (navigate_to, action_open)
    }
}

impl Drop for SpaceScannerApp {
    fn drop(&mut self) {
        // Al cerrar la aplicación o ventana, cancelar de inmediato cualquier hilo de escaneo activo
        self.cancel_token.store(true, Ordering::Relaxed);
    }
}
