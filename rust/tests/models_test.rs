use std::path::PathBuf;
use spacescanner::models::{sort_items_default, ItemType, ScanItem, SortColumn};

fn sample_items() -> Vec<ScanItem> {
    vec![
        ScanItem {
            name: "Zeta".to_string(),
            path: PathBuf::from("/home/user/Zeta"),
            item_type: ItemType::File,
            size_bytes: 1000,
            size_mb: 1.0,
            is_excessive: false,
        },
        ScanItem {
            name: "Alpha".to_string(),
            path: PathBuf::from("/home/user/Alpha"),
            item_type: ItemType::Folder,
            size_bytes: 50000,
            size_mb: 50.0,
            is_excessive: true,
        },
        ScanItem {
            name: "Beta".to_string(),
            path: PathBuf::from("/home/user/Beta"),
            item_type: ItemType::File,
            size_bytes: 200000,
            size_mb: 200.0,
            is_excessive: true,
        },
    ]
}

#[test]
fn test_sort_default() {
    let mut items = vec![
        ScanItem {
            name: "zeta.txt".to_string(),
            path: PathBuf::from("/home/zeta.txt"),
            item_type: ItemType::File,
            size_bytes: 100,
            size_mb: 0.1,
            is_excessive: false,
        },
        ScanItem {
            name: "beta_dir".to_string(),
            path: PathBuf::from("/home/beta_dir"),
            item_type: ItemType::Folder,
            size_bytes: 500,
            size_mb: 0.5,
            is_excessive: false,
        },
        ScanItem {
            name: "alpha_dir".to_string(),
            path: PathBuf::from("/home/alpha_dir"),
            item_type: ItemType::Folder,
            size_bytes: 200,
            size_mb: 0.2,
            is_excessive: false,
        },
        ScanItem {
            name: "alpha.txt".to_string(),
            path: PathBuf::from("/home/alpha.txt"),
            item_type: ItemType::File,
            size_bytes: 50,
            size_mb: 0.05,
            is_excessive: false,
        },
    ];

    sort_items_default(&mut items);

    // Carpetas primero en orden alfabético
    assert_eq!(items[0].name, "alpha_dir");
    assert_eq!(items[0].item_type, ItemType::Folder);
    assert_eq!(items[1].name, "beta_dir");
    assert_eq!(items[1].item_type, ItemType::Folder);

    // Luego archivos en orden alfabético
    assert_eq!(items[2].name, "alpha.txt");
    assert_eq!(items[2].item_type, ItemType::File);
    assert_eq!(items[3].name, "zeta.txt");
    assert_eq!(items[3].item_type, ItemType::File);
}

#[test]
fn test_sort_by_name() {
    let mut items = sample_items();
    SortColumn::Name.sort_items(&mut items, true);
    assert_eq!(items[0].name, "Alpha");
    assert_eq!(items[1].name, "Beta");
    assert_eq!(items[2].name, "Zeta");

    SortColumn::Name.sort_items(&mut items, false);
    assert_eq!(items[0].name, "Zeta");
    assert_eq!(items[1].name, "Beta");
    assert_eq!(items[2].name, "Alpha");
}

#[test]
fn test_sort_by_size() {
    let mut items = sample_items();
    SortColumn::Size.sort_items(&mut items, false); // Mayor a menor
    assert_eq!(items[0].name, "Beta");
    assert_eq!(items[1].name, "Alpha");
    assert_eq!(items[2].name, "Zeta");

    SortColumn::Size.sort_items(&mut items, true); // Menor a mayor
    assert_eq!(items[0].name, "Zeta");
    assert_eq!(items[1].name, "Alpha");
    assert_eq!(items[2].name, "Beta");
}

#[test]
fn test_sort_by_type() {
    let mut items = sample_items();
    SortColumn::Type.sort_items(&mut items, true);
    assert_eq!(items[0].item_type, ItemType::Folder);
}

#[test]
fn test_sort_by_status() {
    let mut items = sample_items();
    SortColumn::Status.sort_items(&mut items, false); // Excesivo primero
    assert!(items[0].is_excessive);
}
