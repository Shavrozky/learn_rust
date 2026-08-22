#[allow(dead_code)]
#[derive(Debug)]

pub enum StatusPesanan {
    MenungguPembayaran,
    Dibayar { nominal: f64 },
    Dikirim(String),
}

pub fn proses_pesanan(status: StatusPesanan) {
    match status {
        StatusPesanan::MenungguPembayaran => println!("[Waiting Menunggu Transfer..."),
        StatusPesanan::Dibayar { nominal} => println!("[PAID] Diterima Rp{}", nominal),
        StatusPesanan::Dikirim(resi) => println!("SHIPPED Resi: {}", resi),
    }
}
