# 🦀 Fullstack Rust Inventory Dashboard

Aplikasi manajemen inventaris sederhana yang dibangun 100% menggunakan bahasa pemrograman Rust, mulai dari server backend hingga antarmuka web (WebAssembly).

## 🛠️ Tech Stack

**Backend (REST API):**
* **[Axum](https://github.com/tokio-rs/axum)** - Web framework *async* yang cepat dan modular.
* **[Redb](https://github.com/cberner/redb)** - *Embedded database* (Key-Value ACID) murni Rust tanpa dependensi eksternal.
* **[Utoipa](https://github.com/juhaku/utoipa)** - Generator dokumentasi OpenAPI / Swagger UI.
* **[Tokio](https://tokio.rs/)** - *Asynchronous runtime*.

**Frontend (Single Page Application):**
* **[Leptos](https://leptos.dev/)** - Framework antarmuka reaktif berbasis WebAssembly (WASM).
* **[Trunk](https://trunkrs.dev/)** - Build tool dan *development server* untuk aplikasi WASM.
* **Tailwind CSS** - Digunakan (via CDN) untuk *styling* komponen UI.

---

## ⚙️ Persyaratan Sistem (Prerequisites)

Pastikan Anda sudah menginstal Rust dan Cargo. Sebelum menjalankan proyek, instal target WebAssembly dan Trunk:

```powershell
# 1. Instal target kompilasi WebAssembly
rustup target add wasm32-unknown-unknown

# 2. Instal Trunk
cargo install --locked trunk