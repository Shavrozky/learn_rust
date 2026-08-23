// File: src/main.rs

mod storage;

use std::io::{self, Write};
use storage::DbStorage;

const DB_FILE: &str = "inventory.redb";

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Gagal membaca input");
    buffer.trim().to_string()
}

fn main() {
    let db = match DbStorage::new(DB_FILE) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Gagal menginisialisasi Database: {}", e);
            return;
        }
    };

    loop {
        println!("\n=== APLIKASI INVENTARIS CRUD (PURE RUST DB) ===");
        println!("1. Lihat Semua Barang (Read)");
        println!("2. Tambah Barang Baru (Create)");
        println!("3. Update Stok / Harga (Update)");
        println!("4. Hapus Barang (Delete)");
        println!("5. Keluar");

        let choice = read_input("Pilih menu [1-5]: ");

        match choice.as_str() {
            "1" => {
                println!("\n--- DAFTAR BARANG ---");
                match db.list_items() {
                    Ok(items) if items.is_empty() => println!("(Database kosong)"),
                    Ok(items) => {
                        for item in items {
                            println!(
                                "ID: {} | Nama: {} | Harga: Rp{:.2} | Stok: {}",
                                item.id, item.name, item.price, item.stock
                            );
                        }
                    }
                    Err(e) => println!("Gagal membaca data: {}", e),
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

                match db.add_item(name, price, stock) {
                    Ok(new_id) => println!("Sukses menambahkan barang dengan ID: {}", new_id),
                    Err(e) => println!("Gagal insert ke DB: {}", e),
                }
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

                match db.update_item(id, new_price, new_stock) {
                    Ok(true) => println!("Barang ID {} berhasil diperbarui!", id),
                    Ok(false) => println!("Barang ID {} tidak ditemukan.", id),
                    Err(e) => println!("Gagal update: {}", e),
                }
            }

            "4" => {
                println!("\n--- HAPUS BARANG ---");
                let id: u32 = match read_input("ID Barang yang ingin dihapus: ").parse() {
                    Ok(v) => v,
                    Err(_) => { println!("ID tidak valid!"); continue; }
                };

                match db.delete_item(id) {
                    Ok(true) => println!("Barang ID {} berhasil dihapus.", id),
                    Ok(false) => println!("Barang ID {} tidak ditemukan.", id),
                    Err(e) => println!("Gagal menghapus: {}", e),
                }
            }

            "5" => {
                println!("Aplikasi ditutup. Data tersimpan di '{}'.", DB_FILE);
                break;
            }

            _ => println!("Pilihan tidak valid, silakan coba lagi."),
        }
    }
}