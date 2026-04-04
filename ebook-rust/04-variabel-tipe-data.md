# Bab 04: Variabel dan Tipe Data

Bayangin meja admin dengan deretan kotak berlabel: "Nama Pelanggan", "Status Tiket", "Prioritas". Tiap kotak menyimpan satu informasi spesifik. Itulah konsep **variabel**: kotak penyimpanan berlabel untuk data. Di Rust, cara mengelola kotak-kotak ini punya aturan yang unik.

[ILUSTRASI: Meja admin dengan deretan kotak berlabel berbeda — "ticket_id", "status", "prioritas". Beberapa kotak terkunci dengan gembok, beberapa tidak.]

---

## Variabel di Rust: Immutable by Default

Di Rust, semua variabel **secara default tidak bisa diubah** setelah pertama kali diisi. Istilah teknisnya: *immutable*. Ini seperti akta kelahiran yang sudah ditandatangani, isinya permanen.

```rust
let ticket_id: u32 = 1;
```

Kalau kamu coba ubah nilainya:

```rust
let ticket_id: u32 = 1;
ticket_id = 2; // ERROR! Tidak bisa diubah
```

Rust langsung protes. Ini bukan bug, ini **fitur**. Banyak bug terjadi karena nilai berubah tanpa disadari. Rust memaksa kamu sadar kapan sebuah nilai boleh berubah.

---

## Mutable Variable dengan `mut`

Tentu ada kebutuhan untuk nilai yang bisa berubah. Status tiket misalnya, bisa berubah dari "open" jadi "closed". Tambahkan kata `mut` (singkatan dari *mutable*) setelah `let`:

```rust
let mut ticket_status = "open";
println!("Status awal: {}", ticket_status);

ticket_status = "closed";
println!("Status sekarang: {}", ticket_status);
```

**Aturan praktis:** mulai tanpa `mut`. Kalau Rust error karena kamu coba ubah nilainya, baru tambahkan `mut`. Jangan langsung pakai `mut` di semua variabel.

---

## Shadowing: Bukan Sama dengan `mut`

Ada teknik lain bernama **shadowing**: bayangkan nulis di whiteboard, lalu di atas tulisan lama kamu tulis ulang. Tulisan lama "tertutup bayangan" tulisan baru.

```rust
let ticket_count = 5;
let ticket_count = ticket_count + 3; // shadowing, bukan mutasi
println!("Jumlah tiket: {}", ticket_count); // Output: 8
```

Bedanya dengan `mut`: dengan `mut`, nilai berubah tapi tipe harus tetap sama. Dengan shadowing, kamu membuat variabel **baru** dengan nama yang sama, bahkan tipenya boleh berbeda.

```rust
let note = 5;                   // tipe: integer
let note = "prioritas tinggi";  // tipe berubah jadi teks — oke di shadowing!
```

Shadowing berguna saat kamu transformasi data tapi tidak mau buat nama variabel baru.

---

## Tipe Data Dasar

Setiap variabel punya **tipe data** (*data type*) yang menentukan jenis dan ukuran datanya di memori.

### Integer

Integer adalah bilangan bulat. Di Rust ada banyak varian:

| Tipe | Ukuran | Rentang nilai |
|------|--------|---------------|
| `i8` | 8-bit | -128 sampai 127 |
| `i32` | 32-bit | sekitar -2 miliar sampai 2 miliar |
| `i64` | 64-bit | sangat besar |
| `u8` | 8-bit | 0 sampai 255 |
| `u32` | 32-bit | 0 sampai ~4 miliar |
| `u64` | 64-bit | 0 sampai sangat besar |
| `usize` | tergantung sistem | dipakai untuk index/panjang koleksi |

Huruf **`i`** = *signed*, bisa positif dan negatif. Huruf **`u`** = *unsigned*, hanya nol dan positif. Angka di belakang (8, 16, 32, 64) adalah ukuran memori dalam **bit**.

Untuk kebanyakan kasus, pakai `i32` (default Rust) atau `u32` untuk nilai yang pasti tidak negatif seperti ID tiket.

```rust
let ticket_id: u32 = 42;
let skor_kepuasan: i32 = -5; // bisa negatif kalau ada komplain ekstrem!
let index: usize = 0;
```

### Float

Float adalah bilangan dengan koma (desimal). Ada dua varian: `f32` untuk presisi standar (32-bit) dan `f64` untuk presisi lebih tinggi (64-bit, default Rust).

```rust
let prioritas: f32 = 8.5;
let response_time: f64 = 1.234567890;
```

Pakai `f64` kalau butuh presisi tinggi, `f32` kalau ingin hemat memori.

### Boolean

Boolean hanya dua nilai: `true` atau `false`. Nama tipenya `bool`.

```rust
let is_open: bool = true;
let is_resolved: bool = false;
```

Boolean cocok untuk status tiket, flag prioritas tinggi, atau apakah user sudah verifikasi email.

### Character

`char` menyimpan **satu karakter**: satu huruf, satu simbol, bahkan satu emoji. Ditulis dengan tanda kutip tunggal `'`. Rust mendukung **Unicode** penuh, artinya bisa menyimpan karakter dari bahasa apapun di dunia.

```rust
let inisial: char = 'A';
let tanda: char = '✓';
let emoji: char = '🎫';
```

Bedakan dengan teks (*string*) yang pakai kutip ganda `"`, itu bab tersendiri.

---

## Type Inference: Rust yang Nebak

[ILUSTRASI: Robot kecil (maskot Rust, Ferris si kepiting) melihat nilai variabel dan memberi label tipe data secara otomatis, seperti mesin pelabel di pabrik.]

Kamu tidak selalu harus tulis tipenya. Rust cukup cerdas untuk **menebak tipe** dari nilai yang dimasukkan, fitur ini disebut *type inference*.

```rust
let title = "Login tidak bisa";  // Rust tahu ini &str (teks, dibahas detail di Bab 05)
let jumlah = 10;                 // Rust tahu ini i32
let aktif = true;                // Rust tahu ini bool
```

Ini bukan berarti Rust tidak punya tipe, justru tipenya sangat ketat, hanya tidak perlu ditulis manual setiap saat. Tulis tipe secara eksplisit kalau tipe tidak bisa ditebak dari konteks, kalau kamu ingin tipe yang berbeda dari default, atau untuk kejelasan kode.

```rust
let harga: f32 = 9.99;   // eksplisit: f32, bukan f64
let id: u32 = 1;         // eksplisit: tidak negatif
```

---

## Konstanta

Konstanta adalah nilai yang **benar-benar tidak pernah berubah sepanjang program berjalan**. Berbeda dengan variabel immutable biasa, konstanta ditulis dengan `const` (bukan `let`), nama ditulis HURUF_BESAR_SEMUA, wajib ada tipe eksplisit, dan bisa dideklarasikan di luar fungsi (global).

```rust
const MAX_TICKETS: u32 = 1000;
const APP_NAME: &str = "Ticket App";  // &str = tipe string pinjaman (dibahas di Bab 05)
```

Konstanta cocok untuk nilai konfigurasi yang tidak boleh berubah: batas maksimal tiket, nama aplikasi, dan sejenisnya.

---

## Contoh: Data Ticket

```rust
fn main() {
    const MAX_TICKETS: u32 = 1000;

    let ticket_id: u32 = 1;
    let title = "Login tidak bisa";  // type inference: &str
    let is_open: bool = true;
    let priority: f32 = 8.5;

    println!("Ticket #{}: {}", ticket_id, title);
    println!("Status terbuka: {}, Prioritas: {}", is_open, priority);
    println!("Batas maksimal ticket: {}", MAX_TICKETS);
}
```

`const MAX_TICKETS` adalah konstanta global yang tidak pernah berubah. `title` tanpa anotasi tipe karena Rust otomatis tahu ini `&str`. `priority` pakai `f32` karena skor bisa desimal. `println!` adalah macro untuk cetak ke layar, `{}` adalah placeholder nilai.

Output yang dihasilkan:

```
Ticket #1: Login tidak bisa
Status terbuka: true, Prioritas: 8.5
Batas maksimal ticket: 1000
```

---

## Latihan

**Tugas 1: Data User**
Deklarasikan variabel-variabel berikut untuk menyimpan data seorang user: `username` bertipe teks, `email` bertipe teks, `role` bertipe teks (contoh: `"admin"` atau `"agent"`), `is_active` bertipe `bool`, dan `total_tickets_handled` bertipe `u32`. Cetak semua data tersebut ke layar dengan `println!`.

**Tugas 2: Hitung Total Tiket**
Mulai dengan variabel `open_tickets: u32 = 15` dan `closed_tickets: u32 = 42`. Gunakan shadowing untuk membuat variabel `total` yang menyimpan jumlah keduanya, lalu cetak hasilnya. Bonus: ubah `open_tickets` menjadi mutable dan tambahkan 5 tiket baru ke dalamnya sebelum dijumlahkan.

---

Di bab berikutnya dibahas **String**: tipe data teks yang lebih lengkap dan punya banyak keunikan di Rust.
