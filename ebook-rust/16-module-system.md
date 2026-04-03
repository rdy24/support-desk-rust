# Bab 16: Module System

Kode yang rapi bukan cuma soal nama variabel yang jelas, tapi juga soal *di mana* kode itu tinggal. Rust mengorganisasi kode lewat **module system**, dan memahaminya sangat penting sebelum proyek mulai membesar.

---

## Kenapa Butuh Modul?

Bayangin meja kerja penuh dokumen berserakan dalam satu tumpukan: kontrak klien, invoice, catatan meeting, desain produk, semua campur aduk jadi satu. Tiap kali mau cari sesuatu, kamu harus korek-korek dari bawah.

Dengan **laci-laci berlabel**: satu untuk kontrak, satu untuk invoice, satu untuk catatan meeting, semuanya langsung ketemu. Itulah fungsi **modul** di Rust.

Tanpa modul, semua struct, fungsi, dan logika bakal tumpuk di satu file. Proyek Support Desk punya user, tiket, autentikasi, database. Kalau semua di `main.rs`, file itu bakal jadi 2000 baris yang menyeramkan.

[ILUSTRASI: Gambar dua meja kerja — kiri berantakan semua kertas campur, kanan rapi dengan laci berlabel "Models", "Handlers", "Services", "DB"]

---

## `mod`: Deklarasi Modul

Kata kunci `mod` mendeklarasikan modul. Ini memberitahu Rust bahwa ada kelompok kode dengan nama tertentu.

```rust
// src/main.rs
mod models;   // Rust akan cari src/models.rs atau src/models/mod.rs
mod handlers; // Rust akan cari src/handlers.rs atau src/handlers/mod.rs
```

Modul juga bisa ditulis inline di dalam file yang sama:

```rust
mod utils {
    pub fn format_email(email: &str) -> String {
        email.to_lowercase()
    }
}
```

Untuk proyek yang lebih besar, file terpisah jauh lebih rapi.

---

## `pub`: Atur Visibilitas

Di Rust, semua hal bersifat private secara default. Struct atau fungsi yang kamu buat tidak bisa diakses dari luar modulnya, kecuali diberi label `pub` (*public*).

```rust
// src/models/user.rs

pub struct User {           // struct bisa diakses dari luar modul
    pub id: u32,            // field ini bisa diakses dari mana saja
    pub name: String,       // field ini juga
    pub email: String,
    pub(crate) role: String, // hanya bisa diakses dalam satu crate (proyek) ini
}

// Fungsi tanpa pub = private, hanya bisa dipakai di dalam file ini
fn internal_helper() {
    // ...
}

// Fungsi dengan pub = bisa dipakai dari modul lain
pub fn create_user(name: String, email: String) -> User {
    User {
        id: 1,
        name,
        email,
        role: String::from("customer"),
    }
}
```

`pub(crate)` adalah tingkat visibilitas di tengah: lebih terbuka dari private, tapi tidak sampai publik ke dunia luar. Berguna untuk hal-hal yang perlu dipakai antar modul dalam proyek, tapi tidak perlu di-*expose* ke pengguna library.

---

## `use`: Import Path

Setelah modul ada, isinya perlu diimport supaya bisa dipakai. Tanpa `use`, kamu harus menulis path lengkap setiap saat:

```rust
// Tanpa use — harus tulis path lengkap setiap saat
let user = crate::models::user::User { ... };

// Dengan use — cukup sebut nama pendeknya
use models::user::User;

let user = User { ... }; // Lebih bersih!
```

Beberapa hal bisa diimport sekaligus:

```rust
use models::user::{User, create_user};
use models::ticket::{Ticket, TicketStatus};
```

Atau pakai `*` untuk import semua, tapi hati-hati karena ini bisa menyebabkan nama bentrok:

```rust
use models::user::*; // Import semua yang pub dari models::user
```

---

## File-Based Module

Ketika `mod models;` ditulis di `main.rs`, Rust akan mencari salah satu dari dua lokasi:

1. `src/models.rs`: satu file tunggal
2. `src/models/mod.rs`: folder dengan file `mod.rs` di dalamnya

Untuk proyek kecil, opsi pertama cukup. Untuk proyek yang lebih besar seperti Support Desk API, opsi kedua lebih sesuai karena `models` punya beberapa bagian: `user`, `ticket`, dan seterusnya.

Di dalam `src/models/mod.rs`, sub-modul dideklarasikan:

```rust
// src/models/mod.rs
pub mod user;   // Rust cari src/models/user.rs
pub mod ticket; // Rust cari src/models/ticket.rs
```

Struktur file mencerminkan struktur modul. Folder di filesystem menjadi hirarki modul di kode.

---

## Struktur Project Support Desk API

Inilah struktur folder yang akan dibangun untuk Support Desk API:

```
src/
├── main.rs
├── models/
│   ├── mod.rs
│   ├── user.rs
│   └── ticket.rs
├── handlers/
│   ├── mod.rs
│   ├── auth.rs
│   └── ticket.rs
├── services/
│   ├── mod.rs
│   └── ticket.rs
└── db/
    ├── mod.rs
    └── repository.rs
```

[ILUSTRASI: Diagram pohon folder di atas dengan panah menunjukkan: "models = laci data", "handlers = laci request HTTP", "services = laci logika bisnis", "db = laci akses database"]

Setiap modul punya tanggung jawab yang jelas. **models** mendefinisikan struct data (User, Ticket). **handlers** menerima request HTTP dan mengirim response. **services** menampung logika bisnis seperti validasi dan aturan bisnis. **db** mengurus query ke database.

Contoh lengkap bagaimana ini terhubung:

```rust
// src/models/user.rs
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub(crate) role: String,
}

// src/models/ticket.rs
pub struct Ticket {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub user_id: u32,
}

// src/models/mod.rs
pub mod user;
pub mod ticket;

// src/main.rs
mod models;
use models::user::User;

fn main() {
    let user = User {
        id: 1,
        name: String::from("Budi"),
        email: String::from("budi@example.com"),
        role: String::from("customer"),
    };
    println!("User: {}", user.name);
}
```

---

## `lib.rs` vs `main.rs`

Di Rust, ada dua titik masuk yang berbeda fungsinya.

**`main.rs`** adalah titik masuk untuk binary, yaitu program yang bisa dijalankan. Di sinilah fungsi `main()` tinggal, dan `cargo run` mulai dari sini.

**`lib.rs`** adalah titik masuk untuk library, yaitu kode yang bisa dipakai proyek lain. Tidak punya `main()`. Digunakan ketika kamu mau share kode atau ketika proyek merupakan gabungan binary dan library.

Untuk Support Desk API, struktur yang umum dipakai:

```
src/
├── main.rs    ← titik masuk, hanya setup dan jalankan server
├── lib.rs     ← semua logika: models, handlers, services, db
```

Dengan cara ini, `main.rs` tetap minimalis:

```rust
// src/main.rs
use support_desk::start_server; // import dari lib.rs

#[tokio::main]
async fn main() {
    start_server().await;
}
```

Dan semua modul lain didaftarkan di `lib.rs`:

```rust
// src/lib.rs
pub mod models;
pub mod handlers;
pub mod services;
pub mod db;

pub async fn start_server() {
    // setup server di sini
}
```

Keuntungannya: kode di `lib.rs` bisa di-test lebih mudah karena bisa diimport sebagai library.

---

## `super::` dan `crate::` untuk Navigasi Path

**`crate::`** mulai dari root proyek, seperti `/` di filesystem. Di mana pun berada, ini selalu merujuk ke root:

```rust
use crate::models::user::User;
use crate::db::repository::UserRepository;
```

**`super::`** naik satu level ke modul parent, seperti `../` di filesystem:

```rust
// src/handlers/auth.rs
// Kita di dalam handlers, mau akses models yang ada di level parent

use super::super::models::user::User;  // naik dua level
// atau lebih baik:
use crate::models::user::User;         // dari root, lebih jelas
```

`super::` paling berguna untuk modul yang deeply nested atau dalam blok `#[cfg(test)]`:

```rust
// src/services/ticket.rs
mod tests {
    use super::*; // import semua dari services::ticket untuk testing

    #[test]
    fn test_create_ticket() {
        // ...
    }
}
```

Sebagai aturan praktis: pakai `crate::` untuk path yang panjang dan jelas, pakai `super::` untuk akses ke parent langsung atau dalam blok `#[cfg(test)]`.

---

## Latihan

Buat struktur modul sederhana:

1. Buat proyek baru: `cargo new belajar-modul`

2. Di `src/`, buat folder `models` dengan dua file:
   - `mod.rs` yang mendaftarkan `pub mod user;`
   - `user.rs` dengan struct `User` yang punya field `id: u32`, `username: String`, dan `is_active: bool`
   - Field `is_active` buat `pub(crate)`, hanya visible dalam proyek

3. Di `main.rs`, import `User` dengan `use` dan buat satu instance, lalu print `username`-nya.

4. **Bonus**: Tambah modul `utils` di dalam `models` (jadi `models::utils`) dengan fungsi `pub fn validate_username(username: &str) -> bool` yang cek apakah username minimal 3 karakter. Panggil fungsi itu dari `main.rs`.

Kalau berhasil compile dan run tanpa error, dasar module system Rust sudah dikuasai.

---

Di bab berikutnya, modul-modul ini akan mulai diisi dengan implementasi nyata, dimulai dari model data untuk tiket support.
