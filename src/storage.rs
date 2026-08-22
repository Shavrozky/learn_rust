// File: src/storage.rs

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub stock: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<Item>,
    next_id: u32,
}

impl Inventory {
    // 1. Muat data dari file JSON (jika file belum ada, buat Inventory kosong)
    pub fn load_from_file(file_path: &str) -> Self {
        if !Path::new(file_path).exists() {
            return Inventory {
                items: Vec::new(),
                next_id: 1,
            };
        }

        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(_) => return Inventory { items: Vec::new(), next_id: 1 },
        };

        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|_| Inventory {
            items: Vec::new(),
            next_id: 1,
        })
    }

    // 2. Simpan seluruh data state ke file JSON
    pub fn save_to_file(&self, file_path: &str) -> Result<(), String> {
        let file = File::create(file_path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);

        // Menulis JSON dengan format rapi (pretty print)
        serde_json::to_writer_pretty(writer, self).map_err(|e| e.to_string())?;
        Ok(())
    }

    // CREATE
    pub fn add_item(&mut self, name: String, price: f64, stock: u32) -> u32 {
        let id = self.next_id;
        let item = Item { id, name, price, stock };
        self.items.push(item);
        self.next_id += 1;
        id
    }

    // READ ALL
    pub fn list_items(&self) -> &[Item] {
        &self.items
    }

    // UPDATE
    pub fn update_item(&mut self, id: u32, new_price: Option<f64>, new_stock: Option<u32>) -> Result<(), String> {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            if let Some(p) = new_price { item.price = p; }
            if let Some(s) = new_stock { item.stock = s; }
            Ok(())
        } else {
            Err(format!("Item dengan ID {} tidak ditemukan.", id))
        }
    }

    // DELETE
    pub fn delete_item(&mut self, id: u32) -> Result<Item, String> {
        if let Some(index) = self.items.iter().position(|item| item.id == id) {
            Ok(self.items.remove(index))
        } else {
            Err(format!("Gagal menghapus: ID {} tidak ditemukan.", id))
        }
    }
}