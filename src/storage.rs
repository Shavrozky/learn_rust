// File: src/storage.rs

use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

const TABLE: TableDefinition<u32, &str> = TableDefinition::new("inventory");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub price: f64,
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

    // CREATE: Tambah item baru
    pub fn add_item(&self, name: String, price: f64, stock: u32) -> Result<u32, Box<dyn std::error::Error>> {
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

        Ok(next_id)
    }

    // READ ALL: Mengambil seluruh data
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

    // UPDATE: Update harga dan stok (Borrow guard di-drop sebelum insert)
    pub fn update_item(&self, id: u32, new_price: Option<f64>, new_stock: Option<u32>) -> Result<bool, Box<dyn std::error::Error>> {
        let write_txn = self.db.begin_write()?;
        let mut table = write_txn.open_table(TABLE)?;

        // 1. Ekstrak data JSON ke struct (peminjaman 'val' selesai di blok ini)
        let item_opt: Option<Item> = if let Some(val) = table.get(id)? {
            Some(serde_json::from_str(val.value())?)
        } else {
            None
        };

        // 2. Modifikasi dan simpan kembali
        if let Some(mut item) = item_opt {
            if let Some(p) = new_price { item.price = p; }
            if let Some(s) = new_stock { item.stock = s; }

            let serialized = serde_json::to_string(&item)?;
            table.insert(id, serialized.as_str())?;
            
            // Drop pinjaman table sebelum commit transaksi
            drop(table);
            write_txn.commit()?;
            Ok(true)
        } else {
            drop(table);
            Ok(false)
        }
    }

    // DELETE: Hapus item berdasarkan ID
    pub fn delete_item(&self, id: u32) -> Result<bool, Box<dyn std::error::Error>> {
        let write_txn = self.db.begin_write()?;
        let mut table = write_txn.open_table(TABLE)?;
        
        let removed = table.remove(id)?.is_some();
        drop(table); // Lepaskan borrow table

        if removed {
            write_txn.commit()?;
        }
        Ok(removed)
    }
}