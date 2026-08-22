// File: src/main.rs

mod storage;

use std::io::{self, Write};
use storage::Inventory;

const DATA_FILE: &str = "inventory.json";

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Gagal membaca input");
    buffer.trim().to_string()
}

fn main() {
    // Muat data lama dari file JSON (jika ada) saat aplikasi dibuka
    let mut inventory = Inventory::load_from_file(DATA_FILE);

    loop {
        println!("\n=== APLIKASI INVENTARIS CRUD (JSON PERSISTENT) ===");
        println!("1. Lihat Semua Barang");
        println!("2. Tambah Barang Baru");
        println!("3. Update Stok / Harga");
        println!("4. Hapus Barang");
        println!("5. Simpan & Keluar");

        let choice = read_input("Pilih menu [1-5]: ");

        match choice.as_str() {
            "1" => {
                println!("\n--- DAFTAR BARANG ---");
                let items = inventory.list_items();
                if items.is_empty() {
                    println!("(Inventaris kosong)");
                } else {
                    for item in items {
                        println!(
                            "ID: {} | Nama: {} | Harga: Rp{:.2} | Stok: {}",
                            item.id, item.name, item.price, item.stock
                        );
                    }
                }
            }

            "2" => {
                println!("\n--- TAMBAH BARANG ---");
                let name = read_input("Nama Barang: ");
                let price: f64 = match read_input("Harga: ").parse() {
                    Ok(v) => v,
                    Err(_) => { println!("Harga tidak valid!"); continue; }
                };
                let stock: u32 = match read_input("Stok: ").parse() {
                    Ok(v) => v,
                    Err(_) => { println!("Stok tidak valid!"); continue; }
                };

                let id = inventory.add_item(name, price, stock);
                // Simpan otomatis ke file setiap ada penambahan
                let _ = inventory.save_to_file(DATA_FILE);
                println!("Sukses menambahkan barang dengan ID: {}", id);
            }

            "3" => {
                println!("\n--- UPDATE BARANG ---");
                let id: u32 = match read_input("ID Barang: ").parse() {
                    Ok(v) => v,
                    Err(_) => { println!("ID tidak valid!"); continue; }
                };

                let p_str = read_input("Harga Baru (Enter jika tidak diubah): ");
                let new_price = if p_str.is_empty() { None } else { p_str.parse().ok() };

                let s_str = read_input("Stok Baru (Enter jika tidak diubah): ");
                let new_stock = if s_str.is_empty() { None } else { s_str.parse().ok() };

                match inventory.update_item(id, new_price, new_stock) {
                    Ok(_) => {
                        let _ = inventory.save_to_file(DATA_FILE);
                        println!("Barang ID {} berhasil diperbarui!", id);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }

            "4" => {
                println!("\n--- HAPUS BARANG ---");
                let id: u32 = match read_input("ID Barang yang ingin dihapus: ").parse() {
                    Ok(v) => v,
                    Err(_) => { println!("ID tidak valid!"); continue; }
                };

                match inventory.delete_item(id) {
                    Ok(item) => {
                        let _ = inventory.save_to_file(DATA_FILE);
                        println!("Barang '{}' (ID: {}) berhasil dihapus.", item.name, item.id);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }

            "5" => {
                if let Err(e) = inventory.save_to_file(DATA_FILE) {
                    eprintln!("Gagal menyimpan data: {}", e);
                } else {
                    println!("Data berhasil disimpan ke '{}'. Sampai jumpa!", DATA_FILE);
                }
                break;
            }

            _ => println!("Pilihan tidak valid, silakan coba lagi."),
        }
    }
}