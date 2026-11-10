use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Folder,
    File,
}

#[derive(Debug, Clone)]
pub struct ScanItem {
    pub name: String,
    pub path: PathBuf,
    pub item_type: ItemType,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub is_excessive: bool,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ScanResult {
    pub path: PathBuf,
    pub items: Vec<ScanItem>,
    pub total_size_bytes: u64,
    pub total_size_mb: f64,
    pub disk_free_bytes: u64,
    pub disk_total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Detailed,
    Visual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Path,
    Type,
    Size,
    Status,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ScanEvent {
    Progress {
        scanned_count: usize,
        current_path: String,
        current_bytes: u64,
    },
    ItemDiscovered(ScanItem),
    Completed(Box<ScanResult>),
    Cancelled,
    Error(String),
}

pub fn sort_items_default(items: &mut [ScanItem]) {
    items.sort_by(|a, b| {
        match (a.item_type == ItemType::Folder, b.item_type == ItemType::Folder) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
}

impl SortColumn {
    pub fn sort_items(&self, items: &mut [ScanItem], ascending: bool) {
        items.sort_by(|a, b| {
            let ord = match self {
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Path => a
                    .path
                    .to_string_lossy()
                    .to_lowercase()
                    .cmp(&b.path.to_string_lossy().to_lowercase()),
                SortColumn::Type => {
                    let type_rank = |t: &ItemType| match t {
                        ItemType::Folder => 0,
                        ItemType::File => 1,
                    };
                    type_rank(&a.item_type)
                        .cmp(&type_rank(&b.item_type))
                        .then_with(|| b.size_bytes.cmp(&a.size_bytes))
                }
                SortColumn::Size => a
                    .size_bytes
                    .cmp(&b.size_bytes)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                SortColumn::Status => {
                    let status_rank = |item: &ScanItem| if item.is_excessive { 1 } else { 0 };
                    status_rank(a)
                        .cmp(&status_rank(b))
                        .then_with(|| b.size_bytes.cmp(&a.size_bytes))
                }
            };

            if ascending {
                ord
            } else {
                ord.reverse()
            }
        });
    }
}
