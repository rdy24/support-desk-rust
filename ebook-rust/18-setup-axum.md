# Bab 18: Setup Project Axum

Fase 2 dimulai di sini. Kalau Fase 1 fokus pada dasar-dasar Rust, Fase 2 adalah tentang membangun sesuatu yang nyata: Support Desk API. Bab ini adalah pondasinya.

[ILUSTRASI: Seorang arsitek sedang meletakkan batu pondasi gedung, dengan blueprint di tangannya. Di latar belakang, gedung masih kosong tapi kerangkanya sudah terlihat.]

Bayangkan kamu mau bangun gedung kantor. Sebelum bisa pasang furnitur, cat dinding, atau pasang lift, kamu perlu pondasi dulu. Pondasi yang kuat menentukan seberapa tinggi gedung itu bisa dibangun. Bab ini adalah pondasi project kita: setup Axum, siapkan semua dependencies, dan jalankan server pertama.

---

## Apa Itu Axum?

Axum adalah **web framework** untuk Rust. Axum yang bertugas mendengarkan request HTTP yang masuk ("hei, ada yang request ke `/users`"), lalu mengarahkannya ke fungsi yang tepat.

Kalau kamu pernah pakai Express.js di Node.js atau Elysia.js di Bun, Axum punya peran yang sama, tapi di dunia Rust.

Yang bikin Axum spesial adalah dua hal:

**Tokio** adalah *async runtime* untuk Rust tempat Axum berjalan. Anggap Tokio sebagai "mesin" yang memungkinkan server kita menangani ribuan koneksi sekaligus tanpa nge-freeze. Jargon *async runtime* artinya: sistem yang mengatur pekerjaan-pekerjaan async (yang butuh waktu nunggu) agar bisa dijalankan secara efisien, bolak-balik, tanpa ada yang nganggur.

**Tower** adalah layer di bawah Axum yang mengurus hal-hal seperti middleware (kode yang berjalan sebelum/sesudah request diproses). Kamu nggak perlu terlalu mikirin Tower untuk sekarang, tapi penting tahu bahwa Axum berdiri di atas ekosistem yang sudah matang.

Kombinasi Axum + Tokio + Tower = web server yang cepat, aman, dan bisa di-extend dengan mudah.

---

## Cargo.toml: Semua Dependencies

Sebelum nulis kode, kita perlu deklarasikan semua "bahan-bahan" yang akan kita pakai. Di Rust, ini dilakukan di file `Cargo.toml`, anggap ini seperti `package.json` di dunia JavaScript.

Kita setup semua dependencies yang dipakai sepanjang project sekarang, bukan nambah satu-satu tiap bab. Tujuannya supaya kamu bisa langsung lihat gambaran besar tool apa saja yang kita pakai.

Buat project baru dulu:

```bash
cargo new support-desk
cd support-desk
```

Lalu buka `Cargo.toml` dan ganti isinya dengan ini:

```toml
[package]
name = "support-desk"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono"] }
jsonwebtoken = "9"
argon2 = "0.5"
validator = { version = "0.18", features = ["derive"] }
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
tower-http = { version = "0.6", features = ["cors"] }

# Semua versi di atas adalah versi saat ebook ini ditulis, Maret 2026
# Cek https://crates.io untuk versi terbaru
```

Berikut ringkasan fungsi masing-masing dependency:

| Dependency | Fungsi |
|---|---|
| `axum` | Web framework utama — router, handler, dll |
| `tokio` | Async runtime — mesin penggerak semua yang async |
| `serde` + `serde_json` | Serialisasi: ubah struct Rust jadi JSON dan sebaliknya |
| `sqlx` | Query ke PostgreSQL dengan type-safety |
| `jsonwebtoken` | Buat dan validasi JWT untuk autentikasi |
| `argon2` | Hash password — simpan password dengan aman |
| `validator` | Validasi input dari user (email valid? password cukup panjang?) |
| `thiserror` | Bikin custom error type dengan mudah |
| `uuid` | Generate ID unik (UUID v4) untuk setiap entitas |
| `chrono` | Handle tanggal dan waktu |
| `dotenvy` | Baca file `.env` untuk konfigurasi |
| `tower-http` | Middleware HTTP: CORS, logging, dll |

Beberapa dependency punya `features = [...]` artinya kita mengaktifkan fitur opsional. Misalnya `sqlx` punya dukungan untuk berbagai database, kita hanya aktifkan yang kita butuhkan (`postgres`, `uuid`, `chrono`) supaya compile time lebih cepat.

---

## Struktur Folder Project

Sebelum mulai coding, kita rencanakan dulu arsitektur folder. Struktur yang baik bikin kode mudah dicari dan mudah dikembangkan orang lain.

```
src/
├── main.rs           ← titik masuk, setup router dan server
├── models/           ← definisi struct data (User, Ticket, dll)
├── handlers/         ← fungsi yang tangani HTTP request
├── services/         ← business logic
├── repositories/     ← query database
├── middleware/       ← auth middleware
└── db.rs             ← setup koneksi database
```

Analoginya seperti restoran. `handlers/` adalah pelayan yang terima pesanan dari tamu (HTTP request). `services/` adalah kepala dapur yang memutuskan proses apa yang perlu dijalankan. `repositories/` adalah asisten dapur yang ambil/simpan bahan dari gudang (database). `models/` adalah daftar menu dan deskripsi bahan-bahan. `middleware/` adalah security di pintu masuk yang mengecek apakah tamu boleh masuk.

Untuk sekarang, kita belum buat semua folder ini. Kita mulai dari `main.rs` dulu, lalu di bab-bab berikutnya kita tambahkan satu per satu seiring fitur berkembang.

---

## Server Pertama

Buka `src/main.rs` dan tulis ini:

```rust
use axum::{routing::get, Router};
use tokio::net::TcpListener;

async fn health_check() -> &'static str {
    "Support Desk API berjalan!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server berjalan di http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
```

Bedah per baris:

**`use axum::{routing::get, Router}`**: kita import `Router` (untuk definisi route) dan `get` (untuk handle GET request).

**`async fn health_check() -> &'static str`**: ini adalah *handler function*. Fungsi ini akan dipanggil ketika ada request ke `/health`. Return type-nya `&'static str` artinya string literal yang hidupnya selama program berjalan. Kenapa `async`? Karena Axum mengharuskan semua handler bersifat async, walau fungsi ini sendiri nggak ada operasi async, ini konvensi yang konsisten.

**`#[tokio::main]`**: ini adalah *attribute macro* yang mengubah `fn main()` biasa menjadi async main yang dijalankan di Tokio runtime. Tanpa ini, kode async kita nggak bisa jalan.

**`Router::new().route("/health", get(health_check))`**: kita buat router baru dan daftarkan route `/health` yang merespons GET request dengan memanggil fungsi `health_check`.

**`TcpListener::bind("0.0.0.0:3000")`**: kita buka port 3000 untuk mendengarkan koneksi. `0.0.0.0` artinya terima koneksi dari semua interface jaringan (bukan hanya localhost).

**`axum::serve(listener, app).await.unwrap()`**: jalankan server. `.await` karena ini operasi async, `.unwrap()` untuk sekarang karena kita belum punya error handling yang proper (akan kita perbaiki nanti).

---

## Jalankan dan Test

Jalankan server:

```bash
cargo run
```

Pertama kali jalan, Cargo akan download dan compile semua dependencies. Butuh beberapa menit. Sabar ya, selanjutnya jauh lebih cepat karena sudah di-cache.

Kalau berhasil, kamu akan lihat:

```
Server berjalan di http://localhost:3000
```

[ILUSTRASI: Terminal dengan output "Server berjalan di http://localhost:3000", dan di sebelahnya browser terbuka menampilkan teks "Support Desk API berjalan!"]

Test dengan `curl` di terminal lain:

```bash
curl http://localhost:3000/health
```

Output yang diharapkan:

```
Support Desk API berjalan!
```

Atau buka browser dan akses `http://localhost:3000/health`, kamu akan lihat teks yang sama.

Kalau mau coba route yang belum ada:

```bash
curl -i http://localhost:3000/blabla
```

Axum otomatis return `404 Not Found`. Kita belum perlu handle ini secara manual.

---

## Apa Selanjutnya?

`Router` yang kita buat di bab ini masih kosong, baru satu route `/health`. Di bab-bab selanjutnya, kita akan terus menambahkan fitur ke router ini: route untuk user, autentikasi, ticket, dan seterusnya. Anggap `Router` ini sebagai "papan pengumuman" yang terus kita tambahin informasi baru.

Struktur `main.rs` juga akan berkembang: kita akan pindahkan route ke file terpisah, tambahkan koneksi database, dan setup middleware CORS.

---

---

## Hasil Akhir Bab Ini

Setelah selesai bab ini, struktur project kamu harus seperti ini:

```
support-desk/
├── Cargo.toml                    ← sudah lengkap semua dependencies
├── Cargo.lock                    ← generated by Cargo
├── src/
│   └── main.rs                   ← server berjalan di port 3000
└── target/                       ← hasil kompilasi (auto-generated)
```

File penting:
- ✅ `Cargo.toml`: semua dependencies sudah declared (axum, tokio, serde, sqlx, dll)
- ✅ `src/main.rs`: server berjalan dengan 1 route `/health`
- ✅ `cargo run`: server bisa dijalankan tanpa error

**Verifikasi:**
```bash
cargo run
# Output: "Server berjalan di http://localhost:3000"

curl http://localhost:3000/health
# Output: "Support Desk API berjalan!"
```

---

## Latihan

Sebelum lanjut ke bab berikutnya, coba ini:

> **Catatan**: Ketiga latihan ini **opsional** untuk melanjutkan ke Bab 19. Tapi sangat disarankan untuk kenyamanan, terutama latihan #3 supaya kamu kenal dengan error message dari Rust compiler.

1. **Tambah route baru:** buat route `/about` yang return string berisi nama kamu. Contoh: `"API dibuat oleh Budi"`. Caranya sama persis seperti `/health`.

2. **Ganti port:** coba ubah port dari `3000` ke `8080`. Jalankan ulang dan test dengan `curl http://localhost:8080/health`. Jangan lupa kembalikan ke `3000` setelah selesai.

3. **Eksplorasi error:** hapus satu baris `use` di bagian atas, lalu coba `cargo build`. Perhatikan pesan error dari Rust compiler. Kembalikan lagi setelah kamu lihat pesannya.

Ketiga latihan ini memang sederhana, tapi tujuannya supaya kamu nyaman dulu dengan siklus: edit kode → jalankan → test. Siklus ini yang akan kita pakai terus sampai akhir project.

---

*Di bab selanjutnya, kita akan mulai menambah route dan memahami handler. Server kamu akan berkembang dari satu endpoint menjadi beberapa endpoint untuk user dan ticket.*
