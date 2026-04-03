# Bab 14: Generics dan Collections

Kita sudah tahu cara bikin struct, enum, dan fungsi. Topik berikutnya: bagaimana kalau satu fungsi atau satu struct bisa bekerja untuk *banyak tipe data sekaligus*? Dan bagaimana cara menyimpan banyak data secara dinamis? Dua topik itulah yang dibahas di bab ini: **Generics** dan **Collections**.

---

## Kotak Pengiriman Universal

[ILUSTRASI: Sebuah kotak kardus besar bertuliskan "ISI APA SAJA" di sampingnya, dengan berbagai barang — buku, helm, laptop — bisa masuk ke dalamnya]

Bayangkan kerja di gudang ekspedisi. Ada dua pilihan kotak: kotak khusus buku yang cuma bisa isi buku, dan kotak universal yang bisa isi buku, helm, laptop, apa pun. Kotak universal jelas lebih fleksibel. Itulah inti dari **Generics** di Rust. Satu "kotak" (fungsi atau struct) yang bisa menampung berbagai tipe data, bukan cuma satu tipe saja.

---

## Generics: Kode yang Bekerja untuk Semua Tipe

**Generics** adalah cara menulis kode yang bisa bekerja untuk berbagai tipe data tanpa harus menulis ulang kodenya berkali-kali.

Tanpa generics, kalau mau fungsi yang bekerja untuk `i32` dan `f64`, kamu harus tulis dua fungsi terpisah:

```rust
fn largest_i32(list: &[i32]) -> &i32 { ... }
fn largest_f64(list: &[f64]) -> &f64 { ... }
```

Dengan generics, cukup satu fungsi:

```rust
fn largest<T>(list: &[T]) -> &T { ... }
```

Huruf `T` di sini adalah **type parameter**, yaitu placeholder untuk tipe yang akan ditentukan nanti saat fungsi dipanggil. Kamu bisa pakai huruf apa saja, tapi konvensinya `T` (singkatan dari *Type*).

---

## Generic Function

Fungsi `largest` ini mencari nilai terbesar dari sebuah list:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    let angka = vec![34, 50, 25, 100, 65];
    println!("Angka terbesar: {}", largest(&angka));

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Huruf terbesar: {}", largest(&chars));
}
```

Perhatikan `T: PartialOrd`. Ini adalah **trait bound**. Artinya: "T boleh tipe apa saja, *asalkan* tipe itu bisa dibandingkan (lebih besar/lebih kecil)." Syarat ini wajar karena kita pakai operator `>` di dalam fungsi. Tanpa trait bound, Rust tidak tahu apakah `T` bisa dibandingkan atau tidak.

---

## Generic Struct

Generics tidak hanya untuk fungsi. Struct juga bisa. Ini sangat berguna untuk bikin respons API yang konsisten:

```rust
#[derive(Debug)]
struct ApiResponse<T> {
    data: T,
    message: String,
    success: bool,
}

impl<T> ApiResponse<T> {
    fn ok(data: T, message: &str) -> Self {
        ApiResponse {
            data,
            message: message.to_string(),
            success: true,
        }
    }
}
```

`ApiResponse<T>` bisa dipakai untuk berbagai jenis data:

```rust
let resp_number = ApiResponse::ok(42, "OK");
let resp_text   = ApiResponse::ok("hello", "OK");
let resp_bool  = ApiResponse::ok(true, "OK");
```

Satu struct, banyak kegunaan.

---

## `Vec<T>`: List Dinamis

**`Vec<T>`** adalah *vector*, yaitu array yang ukurannya bisa berubah secara dinamis. Kalau array biasa ukurannya tetap, `Vec` bisa tumbuh dan menyusut saat program berjalan. Analoginya seperti daftar antrian. Kamu bisa tambah orang baru di belakang, atau lihat berapa banyak yang antri.

```rust
// Cara membuat Vec
let mut tickets: Vec<Ticket> = Vec::new();  // kosong
let angka = vec![1, 2, 3, 4, 5];           // langsung isi pakai macro vec![]

// Tambah elemen
tickets.push(Ticket { id: 1, title: "Login error".to_string(), status: "open".to_string() });

// Akses elemen (hati-hati: bisa panic kalau index tidak ada)
let first = &tickets[0];

// Akses aman pakai .get() — mengembalikan Option<&T>
if let Some(t) = tickets.get(0) {
    println!("Ticket pertama: {}", t.title);
}

// Iterasi
for ticket in &tickets {
    println!("- [#{}] {}", ticket.id, ticket.title);
}

// Panjang list
println!("Total: {}", tickets.len());
```

**`vec![]`** adalah *macro* (perintah spesial di Rust yang diakhiri `!`) untuk membuat Vec dengan isi langsung. Lebih ringkas dari `Vec::new()` lalu push satu-satu.

---

## `HashMap<K, V>`: Kamus Data

**`HashMap<K, V>`** adalah struktur data *key-value*, seperti kamus sungguhan. Kamu cari berdasarkan "kata" (*key*), dan dapat "arti" (*value*).

[ILUSTRASI: Kamus dengan kata di kiri dan definisi di kanan, tapi kata-katanya adalah "open", "closed", "pending" dan nilainya adalah angka jumlah ticket]

Contoh: menyimpan jumlah ticket per status.

```rust
use std::collections::HashMap;

let mut status_count: HashMap<String, u32> = HashMap::new();

// Insert
status_count.insert("open".to_string(), 5);
status_count.insert("closed".to_string(), 10);

// Get — mengembalikan Option<&V>
if let Some(count) = status_count.get("open") {
    println!("Ticket open: {}", count);
}

// Cek apakah key ada
if status_count.contains_key("pending") {
    println!("Ada ticket pending");
}

// Iterasi
for (status, count) in &status_count {
    println!("{}: {} ticket", status, count);
}
```

### Entry API: Cara Elegan Update HashMap

Sering kita mau: "kalau key belum ada, buat dengan nilai awal. Kalau sudah ada, update." Rust punya cara elegan untuk ini:

```rust
let count = status_count.entry("open".to_string()).or_insert(0);
*count += 1;
```

`.entry(key)` cek apakah key ada. `.or_insert(0)` memasukkan nilai 0 kalau belum ada. `*count += 1` menambah nilainya. Tanda `*` untuk *dereference*, karena `count` adalah referensi (`&mut i32`), bukan nilai langsung. Bayangkan `count` seperti alamat rumah — `*count` artinya "isi rumah di alamat ini". Kita perlu `*` untuk mengubah isi, bukan alamatnya.

---

## Gabungan: Filter dan Hitung Ticket

Gabungan semua yang sudah dipelajari dalam satu program Support Desk:

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct ApiResponse<T> {
    data: T,
    message: String,
    success: bool,
}

impl<T> ApiResponse<T> {
    fn ok(data: T, message: &str) -> Self {
        ApiResponse {
            data,
            message: message.to_string(),
            success: true,
        }
    }
}

#[derive(Debug)]
struct Ticket {
    id: u32,
    title: String,
    status: String,
}

fn main() {
    // Vec<Ticket>
    let mut tickets: Vec<Ticket> = Vec::new();
    tickets.push(Ticket { id: 1, title: "Login error".to_string(), status: "open".to_string() });
    tickets.push(Ticket { id: 2, title: "Server down".to_string(), status: "open".to_string() });
    tickets.push(Ticket { id: 3, title: "Lupa password".to_string(), status: "closed".to_string() });

    println!("Total ticket: {}", tickets.len());
    for ticket in &tickets {
        println!("- [#{}] {}", ticket.id, ticket.title);
    }

    // Filter ticket yang masih open
    let open_tickets: Vec<&Ticket> = tickets.iter()
        .filter(|t| t.status == "open")
        .collect();
    println!("Ticket terbuka: {}", open_tickets.len());

    // ⚠️ Catatan: .iter(), .filter(), dan .collect() adalah bagian dari *iterator pattern* yang akan dibahas detail di Bab 15. 
    // Untuk sekarang, cukup pahami bahwa ini adalah cara idiomatis Rust untuk memproses koleksi data.

    // Hitung ticket per status dengan HashMap
    let mut ticket_count_by_status: HashMap<String, u32> = HashMap::new();
    for ticket in &tickets {
        let count = ticket_count_by_status.entry(ticket.status.clone()).or_insert(0);
        *count += 1;
    }

    for (status, count) in &ticket_count_by_status {
        println!("Status '{}': {} ticket", status, count);
    }

    // Bungkus dengan generic ApiResponse
    let response = ApiResponse::ok(tickets.len(), "Berhasil mengambil data ticket");
    println!("{:?}", response);
}
```

Output yang diharapkan:

```
Total ticket: 3
- [#1] Login error
- [#2] Server down
- [#3] Lupa password
Ticket terbuka: 2
Status 'open': 2 ticket
Status 'closed': 1 ticket
ApiResponse { data: 3, message: "Berhasil mengambil data ticket", success: true }
```

Satu `ApiResponse<T>` yang fleksibel, `Vec<Ticket>` untuk menyimpan banyak ticket, dan `HashMap` untuk statistik per status. Semua dalam satu program yang rapi.

---

## Latihan

1. **Latihan Vec**: Tambahkan 2 ticket baru dengan status `"pending"` ke dalam `tickets`. Lalu cetak semua ticket yang statusnya bukan `"closed"`.

2. **Latihan HashMap**: Buat `HashMap<u32, String>` yang menyimpan `id` ticket sebagai key dan `title` sebagai value. Coba akses title dari ticket dengan id `2`.

3. **Latihan Generic**: Buat generic function `fn first<T>(list: &[T]) -> Option<&T>` yang mengembalikan elemen pertama dari sebuah slice, atau `None` kalau list kosong. Coba panggil dengan `Vec<Ticket>` dan `Vec<i32>`.

4. **Tantangan**: Tambahkan field `priority: u8` ke struct `Ticket` (1 = rendah, 3 = tinggi). Gunakan `HashMap<u8, Vec<&Ticket>>` untuk mengelompokkan ticket berdasarkan priority.

---

> **Catatan versi**: Fitur generics dan collections (`Vec`, `HashMap`) sudah stabil sejak awal Rust dan tidak berubah secara signifikan. Contoh kode di bab ini kompatibel dengan Rust stable terbaru (saat ebook ini ditulis, Maret 2026).
