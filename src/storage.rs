// File: src/storage.rs

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub stock: u32,
}

pub struct Inventory {
    items: Vec<Item>,
    next_id: u32,
}

impl Inventory {
    // Constructor untuk inisialisasi struct
    pub fn new() -> Self {
        Inventory {
            items: Vec::new(),
            next_id: 1,
        }
    }

    // 1. CREATE: Menambahkan barang baru
    pub fn add_item(&mut self, name: String, price: f64, stock: u32) -> u32 {
        let id = self.next_id;
        let item = Item {
            id,
            name,
            price,
            stock,
        };
        self.items.push(item);
        self.next_id += 1;
        id
    }

    // 2. READ ALL: Meminjam slice dari seluruh item (&[Item])
    pub fn list_items(&self) -> &[Item] {
        &self.items
    }

    // 3. READ ONE: Mencari item berdasarkan ID menggunakan Option
    pub fn find_by_id(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    // 4. UPDATE: Mengubah data barang (harga & stok)
    pub fn update_item(&mut self, id: u32, new_price: Option<f64>, new_stock: Option<u32>) -> Result<(), String> {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            if let Some(p) = new_price {
                item.price = p;
            }
            if let Some(s) = new_stock {
                item.stock = s;
            }
            Ok(())
        } else {
            Err(format!("Item dengan ID {} tidak ditemukan.", id))
        }
    }

    // 5. DELETE: Menghapus item berdasarkan ID
    pub fn delete_item(&mut self, id: u32) -> Result<Item, String> {
        if let Some(index) = self.items.iter().position(|item| item.id == id) {
            let removed = self.items.remove(index);
            Ok(removed)
        } else {
            Err(format!("Gagal menghapus: ID {} tidak ditemukan.", id))
        }
    }
}