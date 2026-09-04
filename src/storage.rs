use redb::{Database, Error, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

const ITEMS_TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("items");
const LOGS_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("audit_logs");

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub stock: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AuditLog {
    pub id: u64,
    pub item_id: u32,
    pub change_type: String,
    pub delta: i32,
    pub timestamp: u64,
}

pub struct DbStorage {
    db: Database,
}

impl DbStorage {
    pub fn new(path: &str) -> Result<Self, Error> {
        let db = Database::create(path)?;
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(ITEMS_TABLE)?;
            let _ = write_txn.open_table(LOGS_TABLE)?;
        }
        write_txn.commit()?;
        Ok(Self { db })
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn next_log_id() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64
    }

    pub fn list_items(&self) -> Result<Vec<Item>, Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ITEMS_TABLE)?;
        let mut items = Vec::new();

        for item_res in table.iter()? {
            let (_, value) = item_res?;
            let item: Item = serde_json::from_slice(value.value())
                .map_err(|_| Error::Corrupted("JSON error".into()))?;
            items.push(item);
        }
        items.sort_by_key(|i| i.id);
        Ok(items)
    }

    pub fn add_item(&self, name: String, price: f64, stock: u32) -> Result<Item, Error> {
        let write_txn = self.db.begin_write()?;
        let mut items_table = write_txn.open_table(ITEMS_TABLE)?;
        let mut logs_table = write_txn.open_table(LOGS_TABLE)?;

        let next_id = items_table
            .iter()?
            .last()
            .map(|res| res.map(|(k, _)| k.value() + 1))
            .unwrap_or(Ok(1))?;

        let new_item = Item {
            id: next_id,
            name,
            price,
            stock,
        };
        let serialized = serde_json::to_vec(&new_item).unwrap();
        items_table.insert(next_id, serialized.as_slice())?;

        let log = AuditLog {
            id: Self::next_log_id(),
            item_id: next_id,
            change_type: "Created".to_string(),
            delta: stock as i32,
            timestamp: Self::current_timestamp(),
        };
        let log_bytes = serde_json::to_vec(&log).unwrap();
        logs_table.insert(log.id, log_bytes.as_slice())?;

        drop(items_table);
        drop(logs_table);
        write_txn.commit()?;
        Ok(new_item)
    }

    pub fn update_item(
        &self,
        id: u32,
        price: Option<f64>,
        stock: Option<u32>,
    ) -> Result<Option<Item>, Error> {
        let write_txn = self.db.begin_write()?;
        let mut items_table = write_txn.open_table(ITEMS_TABLE)?;
        let mut logs_table = write_txn.open_table(LOGS_TABLE)?;

        // 1. Ekstrak data ke struct terpisah agar borrow guard AccessGuard selesai di statement ini
        let existing_item: Option<Item> = match items_table.get(id)? {
            Some(guard) => {
                let item: Item = serde_json::from_slice(guard.value())
                    .map_err(|_| Error::Corrupted("JSON error".into()))?;
                Some(item)
            }
            None => None,
        };

        // 2. Sekarang items_table bebas dipinjam secara mutable (.insert)
        let result = if let Some(mut item) = existing_item {
            if let Some(new_stock) = stock {
                let delta = new_stock as i32 - item.stock as i32;
                if delta != 0 {
                    let log = AuditLog {
                        id: Self::next_log_id(),
                        item_id: id,
                        change_type: "StockAdjusted".to_string(),
                        delta,
                        timestamp: Self::current_timestamp(),
                    };
                    let log_bytes = serde_json::to_vec(&log).unwrap();
                    logs_table.insert(log.id, log_bytes.as_slice())?;
                }
                item.stock = new_stock;
            }

            if let Some(p) = price {
                item.price = p;
            }

            let serialized = serde_json::to_vec(&item).unwrap();
            items_table.insert(id, serialized.as_slice())?;
            Some(item)
        } else {
            None
        };

        drop(items_table);
        drop(logs_table);
        write_txn.commit()?;
        Ok(result)
    }

    pub fn delete_item(&self, id: u32) -> Result<bool, Error> {
        let write_txn = self.db.begin_write()?;
        let mut items_table = write_txn.open_table(ITEMS_TABLE)?;
        let mut logs_table = write_txn.open_table(LOGS_TABLE)?;

        // 1. Ekstrak item yang dihapus dan lepas guard AccessGuard
        let removed_item: Option<Item> = match items_table.remove(id)? {
            Some(guard) => {
                let item: Item = serde_json::from_slice(guard.value())
                    .map_err(|_| Error::Corrupted("JSON error".into()))?;
                Some(item)
            }
            None => None,
        };

        // 2. Tulis log audit
        let exists = if let Some(item) = removed_item {
            let log = AuditLog {
                id: Self::next_log_id(),
                item_id: id,
                change_type: "Deleted".to_string(),
                delta: -(item.stock as i32),
                timestamp: Self::current_timestamp(),
            };
            let log_bytes = serde_json::to_vec(&log).unwrap();
            logs_table.insert(log.id, log_bytes.as_slice())?;
            true
        } else {
            false
        };

        drop(items_table);
        drop(logs_table);
        write_txn.commit()?;
        Ok(exists)
    }

    pub fn list_logs(&self) -> Result<Vec<AuditLog>, Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(LOGS_TABLE)?;
        let mut logs = Vec::new();

        for log_res in table.iter()? {
            let (_, value) = log_res?;
            let log: AuditLog = serde_json::from_slice(value.value())
                .map_err(|_| Error::Corrupted("JSON error".into()))?;
            logs.push(log);
        }
        logs.reverse();
        Ok(logs)
    }
}