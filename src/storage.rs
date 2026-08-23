// File: src/storage.rs

use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const TABLE: TableDefinition<u32, &str> = TableDefinition::new("inventory");

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Item {
    #[schema(example = 1)]
    pub id: u32,
    #[schema(example = "Keyboard Mechanical RGB")]
    pub name: String,
    #[schema(example = 850000.0)]
    pub price: f64,
    #[schema(example = 12)]
    pub stock: u32,
}

pub struct DbStorage {
    db: Database,
}

impl DbStorage {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::create(db_path)?;
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(TABLE)?;
        }
        write_txn.commit()?;
        Ok(DbStorage { db })
    }

    pub fn add_item(&self, name: String, price: f64, stock: u32) -> Result<Item, Box<dyn std::error::Error>> {
        let items = self.list_items()?;
        let next_id = items.iter().map(|i| i.id).max().unwrap_or(0) + 1;

        let item = Item { id: next_id, name, price, stock };
        let json_data = serde_json::to_string(&item)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(next_id, json_data.as_str())?;
        }
        write_txn.commit()?;

        Ok(item)
    }

    pub fn list_items(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        
        let mut items = Vec::new();
        for result in table.iter()? {
            let (_, value) = result?;
            let item: Item = serde_json::from_str(value.value())?;
            items.push(item);
        }

        items.sort_by_key(|i| i.id);
        Ok(items)
    }

    pub fn update_item(&self, id: u32, new_price: Option<f64>, new_stock: Option<u32>) -> Result<Option<Item>, Box<dyn std::error::Error>> {
        let write_txn = self.db.begin_write()?;
        let mut table = write_txn.open_table(TABLE)?;

        let item_opt: Option<Item> = if let Some(val) = table.get(id)? {
            Some(serde_json::from_str(val.value())?)
        } else {
            None
        };

        if let Some(mut item) = item_opt {
            if let Some(p) = new_price { item.price = p; }
            if let Some(s) = new_stock { item.stock = s; }

            let serialized = serde_json::to_string(&item)?;
            table.insert(id, serialized.as_str())?;
            drop(table);
            write_txn.commit()?;
            Ok(Some(item))
        } else {
            drop(table);
            Ok(None)
        }
    }

    pub fn delete_item(&self, id: u32) -> Result<bool, Box<dyn std::error::Error>> {
        let write_txn = self.db.begin_write()?;
        let mut table = write_txn.open_table(TABLE)?;
        let removed = table.remove(id)?.is_some();
        drop(table);

        if removed {
            write_txn.commit()?;
        }
        Ok(removed)
    }
}