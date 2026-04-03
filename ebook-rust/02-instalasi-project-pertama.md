# Bab 02: Instalasi dan Project Pertama

Bab ini adalah fondasi paling konkret sebelum kita bisa mulai ngoding: **setup lingkungan kerja**. Ibarat mau masak, kita perlu dapur dulu sebelum bisa bikin makanan. Di sini kita install Rust, buat project pertama, dan jalankan kode pertama kita.

---

## Instalasi Rust

### Apa itu rustup?

**rustup** adalah satu installer yang sekaligus mengunduh dan mengelola semua alat yang dibutuhkan untuk ngoding Rust. Dengan satu perintah, kamu dapat **rustc** (compiler yang menerjemahkan kode ke program yang bisa dijalankan), **cargo** (package manager + build tool), dan **rustfmt** serta **clippy** (alat tambahan untuk merapikan dan mengecek kode).

[ILUSTRASI: Kotak toolbox berlabel "rustup" yang isinya tiga alat: rustc, cargo, dan clippy — masing-masing dengan label singkat fungsinya]

### Cara Install

**Linux / macOS:**

Buka terminal, lalu jalankan:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Setelah selesai, ikuti instruksi di layar (biasanya tinggal tekan Enter). Lalu jalankan:

```bash
source $HOME/.cargo/env
```

Perintah ini memberitahu terminal di mana letak alat-alat Rust yang baru diinstall.

**Windows:**

Unduh installer dari [https://rustup.rs](https://rustup.rs), lalu jalankan file `.exe`-nya. Ikuti wizard instalasi seperti install aplikasi biasa.

> **Catatan:** Di Windows, kamu mungkin perlu install **Visual Studio Build Tools** juga. Installer rustup akan memberi tahu kalau memang diperlukan.

### Verifikasi Instalasi

Setelah install selesai, cek apakah semuanya berjalan dengan baik. Buka terminal baru, lalu ketik:

```bash
rustc --version
```

Kalau berhasil, outputnya kira-kira seperti ini:

```
rustc 1.8x.x (xxxxxxxx YYYY-MM-DD)
```

Sekarang cek cargo:

```bash
cargo --version
```

Output:

```
cargo 1.8x.x (xxxxxxxx YYYY-MM-DD)
```

Versi yang kamu lihat pasti berbeda dari contoh di atas, tidak masalah. Rust rilis versi baru setiap 6 minggu. Yang penting tidak ada pesan error. Kalau muncul *"command not found"*, coba restart terminal dan ulangi.

---

## Membuat Project Pertama

### Cargo sebagai Manajer Project

Kalau rustup itu toolbox-nya, maka **cargo** itu asisten pribadimu. Cargo yang ngurus pembuatan struktur folder project baru, pengunduhan library tambahan, proses kompilasi (build), dan menjalankan program. Anggap saja cargo seperti asisten kontraktor: kamu tinggal bilang "buatkan proyek baru" atau "bangun sekarang", dan dia yang eksekusi.

### Membuat Project dengan `cargo new`

Di terminal, pergi ke folder tempat kamu menyimpan project (misalnya `~/projects`), lalu jalankan:

```bash
cargo new support-desk
```

Perintah ini membuat folder baru bernama `support-desk` dengan struktur awal yang sudah siap pakai.

Kamu akan melihat output seperti:

```
Created binary (application) `support-desk` package
```

Masuk ke folder project:

```bash
cd support-desk
```

### Struktur Folder Hasil `cargo new`

```
support-desk/
├── Cargo.toml
└── src/
    └── main.rs
```

| Nama | Fungsi |
|------|--------|
| `Cargo.toml` | File konfigurasi project (nama, versi, daftar library) |
| `src/` | Folder tempat semua kode Rust kita tinggal |
| `src/main.rs` | File kode utama, titik awal program kita dijalankan |

[ILUSTRASI: Diagram pohon folder support-desk dengan tanda panah menunjuk ke Cargo.toml (berlabel "Konfigurasi"), folder src (berlabel "Kode kita"), dan main.rs (berlabel "Titik masuk program")]

Cargo sudah menyiapkan "kerangka rumah" kita. Tinggal ngisi isinya.

---

## Kenalan dengan Cargo.toml

Buka file `Cargo.toml`, ini adalah file konfigurasi project. Isinya seperti ini:

```toml
[package]
name = "support-desk"
version = "0.1.0"
edition = "2024"

[dependencies]
```

**`[package]`** mendefinisikan identitas project. Seperti KTP-nya project. `name` adalah nama project (sekaligus nama file program yang dihasilkan), `version` mengikuti aturan **semver** (semantic versioning) dengan format major.minor.patch, dan `edition = "2024"` adalah versi "dialek" Rust yang dipakai, edisi 2024 adalah yang terbaru dan paling direkomendasikan.

**`[dependencies]`** adalah tempat mendaftarkan library tambahan yang dibutuhkan project. Sekarang masih kosong. Di bab-bab berikutnya, kita akan menambahkan library seperti `axum` untuk web server, `sqlx` untuk database, dan lain-lain. Cukup ditambahkan di bagian ini.

> **Tips:** TOML adalah format file konfigurasi yang mudah dibaca manusia. Mirip seperti `.env` tapi lebih terstruktur. Bagian yang diawali `[nama]` disebut *section* atau *tabel*.

---

## Hello World!

Buka file `src/main.rs`. Cargo sudah menyiapkan kode Hello World untuk kita, tapi ganti isinya dengan yang lebih sesuai project kita:

```rust
fn main() {
    println!("Halo, Support Desk!");
}
```

### Apa artinya kode ini?

**`fn main()`**: `fn` adalah singkatan dari *function* (fungsi). `main` adalah nama khusus yang Rust cari saat program dijalankan, itulah pintu masuk program. `()` artinya fungsi ini tidak menerima input apapun, dan `{ ... }` berisi perintah-perintah yang akan dijalankan.

**`println!("Halo, Support Desk!")`**: `println` mencetak teks ke layar diikuti baris baru. Tanda seru `!` di belakangnya menandakan ini adalah **macro**, bukan fungsi biasa. Macro itu seperti "shortcut" yang Rust expand menjadi kode yang lebih panjang saat kompilasi. Untuk sekarang, anggap saja `println!` sebagai perintah print yang sedikit "ajaib".

---

## Jalankan Program: `cargo run`

Di terminal, pastikan kamu berada di dalam folder `support-desk`, lalu ketik:

```bash
cargo run
```

Kamu akan melihat output seperti ini:

```
   Compiling support-desk v0.1.0 (/Users/kamu/projects/support-desk)
    Finished dev [unoptimized + debuginfo] target(s) in 1.23s
     Running `target/debug/support-desk`
Halo, Support Desk!
```

Baris terakhir `Halo, Support Desk!` adalah output program kita.

### `cargo run` vs `cargo build`: Bedanya di Sini

| Perintah | Yang Terjadi |
|----------|--------------|
| `cargo build` | Hanya **mengompilasi** kode menjadi file program, tapi tidak menjalankannya. File program tersimpan di `target/debug/` |
| `cargo run` | **Mengompilasi** kode, lalu langsung **menjalankan** hasilnya |

Analoginya: `cargo build` itu masak makanan sampai matang lalu disimpan di kulkas. `cargo run` itu masak langsung makan di tempat. Pakai `cargo run` untuk development sehari-hari karena lebih praktis, dan `cargo build` saat mau mendistribusikan program ke orang lain atau deploy ke server.

> **Tambahan:** Untuk versi final yang siap rilis, gunakan `cargo build --release`. Hasilnya lebih kecil dan lebih cepat karena dioptimasi penuh, tapi proses kompilasi lebih lama.

---

## Ke Depannya: Menambah Dependencies

Sekarang `[dependencies]` di `Cargo.toml` masih kosong. Tapi seiring kita membangun aplikasi Support Desk, kita akan menambahkan library-library yang dibutuhkan.

Contohnya nanti akan terlihat seperti ini (versi yang dipakai saat ebook ini ditulis, Maret 2026, cek [crates.io](https://crates.io) untuk versi terbaru saat kamu membaca ini):

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-native-tls"] }
```

Setiap kali kita tambahkan baris baru di sana dan jalankan `cargo build`, Cargo akan otomatis mengunduh library tersebut dari internet.

---

## Latihan

Sebelum lanjut ke bab berikutnya, coba kerjakan dua latihan kecil ini:

**Latihan 1: Jalankan Hello World:**
Pastikan kamu berhasil menjalankan `cargo run` dan melihat output `Halo, Support Desk!` di terminal. Kalau belum berhasil, baca ulang bagian instalasi.

**Latihan 2: Ubah Pesan:**
Buka `src/main.rs`, ganti teks `"Halo, Support Desk!"` dengan nama kamu sendiri, misalnya `"Halo, saya Budi dan saya belajar Rust!"`. Simpan file, lalu jalankan `cargo run` lagi. Pastikan teks baru muncul di layar.

---

Di bab berikutnya, kita akan mulai merancang struktur aplikasi Support Desk dan belajar konsep-konsep dasar Rust yang akan sering dipakai. Sampai jumpa!
