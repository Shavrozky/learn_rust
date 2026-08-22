// File: src/main.rs

mod storage;

use std::io::{self, Write};
use storage::Inventory;

// Helper function untuk membaca input teks dari keyboard
fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Gagal membaca input");
    
    buffer.trim().to_string()
}

fn main() {
    let mut inventory = Inventory::new();

    // Data awal (Seed dummy data)
    inventory.add_item(String::from("Laptop Asus"), 12000000.0, 5);
    inventory.add_item(String::from("Mouse Wireless"), 150000.0, 20);

    loop {
        println!("\n=== APLIKASI INVENTARIS CRUD ===");
        println!("1. Lihat Semua Barang (Read)");
        println!("2. Tambah Barang Baru (Create)");
        println!("3. Update Stok / Harga (Update)");
        println!("4. Hapus Barang (Delete)");
        println!("5. Keluar");

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
                    Ok(val) => val,
                    Err(_) => {
                        println!("Input harga tidak valid!");
                        continue;
                    }
                };

                let stock: u32 = match read_input("Stok: ").parse() {
                    Ok(val) => val,
                    Err(_) => {
                        println!("Input stok tidak valid!");
                        continue;
                    }
                };

                let new_id = inventory.add_item(name, price, stock);
                println!("Sukses menambahkan barang dengan ID: {}", new_id);
            }

            "3" => {
                println!("\n--- UPDATE BARANG ---");
                let id_input = read_input("Masukkan ID Barang yang ingin diubah: ");
                let id: u32 = match id_input.parse() {
                    Ok(val) => val,
                    Err(_) => {
                        println!("ID tidak valid!");
                        continue;
                    }
                };

                let new_price_str = read_input("Harga Baru (kosongkan jika tidak diubah): ");
                let new_price: Option<f64> = if new_price_str.is_empty() {
                    None
                } else {
                    new_price_str.parse().ok()
                };

                let new_stock_str = read_input("Stok Baru (kosongkan jika tidak diubah): ");
                let new_stock: Option<u32> = if new_stock_str.is_empty() {
                    None
                } else {
                    new_stock_str.parse().ok()
                };

                match inventory.update_item(id, new_price, new_stock) {
                    Ok(_) => println!("Barang dengan ID {} berhasil diperbarui!", id),
                    Err(err) => println!("Error: {}", err),
                }
            }

            "4" => {
                println!("\n--- HAPUS BARANG ---");
                let id_input = read_input("Masukkan ID Barang yang ingin dihapus: ");
                let id: u32 = match id_input.parse() {
                    Ok(val) => val,
                    Err(_) => {
                        println!("ID tidak valid!");
                        continue;
                    }
                };

                match inventory.delete_item(id) {
                    Ok(deleted) => println!("Barang '{}' (ID: {}) berhasil dihapus.", deleted.name, deleted.id),
                    Err(err) => println!("Error: {}", err),
                }
            }

            "5" => {
                println!("Keluar dari aplikasi...");
                break;
            }

            _ => println!("Pilihan tidak valid, silakan coba lagi."),
        }
    }
}