use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use spacescanner::models::ScanEvent;
use spacescanner::scanner::{parse_size_limit, start_scan_thread};

#[test]
fn test_parse_size_limit() {
    assert_eq!(parse_size_limit("5000 MB"), 5000.0);
    assert_eq!(parse_size_limit("1 GB"), 1024.0);
    assert_eq!(parse_size_limit("2 TB"), 2.0 * 1024.0 * 1024.0);
    assert_eq!(parse_size_limit("512 KB"), 0.5);
    assert_eq!(parse_size_limit(""), 5000.0);
}

#[test]
fn test_scan_directory_isolated() {
    let temp_dir = std::env::temp_dir().join("spacescanner_unit_test");
    let _ = fs::remove_dir_all(&temp_dir);
    let sub1 = temp_dir.join("folder_a");
    let sub2 = temp_dir.join("folder_b");
    fs::create_dir_all(&sub1).unwrap();
    fs::create_dir_all(&sub2).unwrap();
    fs::write(temp_dir.join("root_file.txt"), vec![0u8; 1024]).unwrap();
    fs::write(sub1.join("file1.bin"), vec![0u8; 2048]).unwrap();
    fs::write(sub2.join("file2.bin"), vec![0u8; 4096]).unwrap();

    let (sender, receiver) = channel();
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let cancel = Arc::new(AtomicBool::new(false));
    start_scan_thread(
        temp_dir.clone(),
        1.0,
        5,
        cache,
        cancel,
        sender,
    );

    let mut completed = false;
    while let Ok(event) = receiver.recv() {
        match &event {
            ScanEvent::Completed(r) => {
                assert_eq!(r.items.len(), 3); // root_file.txt, folder_a, folder_b
                assert_eq!(r.total_size_bytes, 1024 + 2048 + 4096);
                completed = true;
                break;
            }
            ScanEvent::Cancelled => panic!("Cancelled unexpectedly"),
            ScanEvent::Error(e) => panic!("Error during scan: {}", e),
            _ => {}
        }
    }
    assert!(completed);
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_scan_aimodels() {
    let path = PathBuf::from("/home/joss/aimodels");
    if !path.exists() {
        // Omitir en entornos CI como GitHub Actions donde esta ruta local no existe
        return;
    }

    let (sender, receiver) = channel();
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let cancel = Arc::new(AtomicBool::new(false));
    start_scan_thread(
        path,
        5000.0,
        7,
        cache,
        cancel,
        sender,
    );
    while let Ok(event) = receiver.recv() {
        match &event {
            ScanEvent::Completed(r) => {
                assert_eq!(r.items.len(), 4);
                assert!(r.total_size_mb > 8000.0);
                break;
            }
            ScanEvent::Cancelled => panic!("Cancelled unexpectedly"),
            ScanEvent::Error(e) => panic!("Error during scan: {}", e),
            _ => {}
        }
    }
}
