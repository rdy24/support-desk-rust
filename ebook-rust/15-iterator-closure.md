# Bab 15: Iterator dan Closure

Dua fitur andalan Rust yang dibahas di bab ini adalah **closure** dan **iterator**. Keduanya sering jalan berdampingan, dan setelah paham, kamu bakal jarang nulis loop manual lagi.

[ILUSTRASI: conveyor belt pabrik — ban berjalan yang membawa barang dari satu stasiun ke stasiun berikutnya, tiap stasiun melakukan satu pekerjaan spesifik sebelum barang diteruskan]

Bayangkan sebuah pabrik dengan ban berjalan (conveyor belt). Barang masuk dari ujung kiri, lalu melewati beberapa stasiun: stasiun pertama menyaring barang cacat, stasiun kedua mengecat barang, stasiun ketiga membungkus. Hasil akhirnya dikumpulkan di ujung kanan. Itulah cara kerja iterator + closure di Rust. Data "jalan" melewati beberapa tahap pemrosesan, dan kamu yang tentukan apa yang terjadi di tiap tahap.

---

## Closure: Fungsi Mini

**Closure** adalah fungsi anonim (tanpa nama) yang bisa kamu tulis langsung di tempat kamu butuhkan. Bedanya dengan fungsi biasa: closure bisa "mengambil" (capture) variabel dari luar dirinya.

Syntax closure di Rust:

```rust
// Closure sederhana — parameter diapit tanda pipe |...|
let tambah = |x| x + 1;
println!("{}", tambah(5)); // 6

// Closure dengan blok
let sapa = |nama| {
    let pesan = format!("Halo, {}!", nama);
    pesan
};
println!("{}", sapa("Budi")); // Halo, Budi!

// Capture variabel dari luar — ini yang bikin closure spesial
let threshold = 8;
let is_urgent = |priority: u8| priority >= threshold;
// threshold "dibawa" masuk ke dalam closure
```

Bedanya dengan fungsi biasa (`fn`): fungsi biasa tidak bisa pakai variabel dari luar tanpa dikirim sebagai parameter. Closure bisa langsung "memeluk" variabel sekitarnya.

---

## Iterator: Jalan Satu per Satu

**Iterator** adalah sesuatu yang bisa kamu minta "kasih satu elemen berikutnya" berulang kali sampai habis. Di Rust, iterator mengimplementasikan **trait** bernama `Iterator`. Ada method `.next()` yang mengembalikan elemen berikutnya.

Yang menarik: iterator di Rust bersifat **lazy**. Mereka tidak langsung memproses semua data sekaligus, tapi baru bekerja ketika kamu "meminta hasilnya". Ini efisien karena tidak buang-buang memori untuk data yang belum perlu.

Ada tiga cara membuat iterator dari sebuah koleksi:

```rust
let tickets = vec![/* ... */];

tickets.iter()        // Pinjam data (borrow), elemen bertipe &Ticket
tickets.into_iter()   // Ambil kepemilikan (consume), elemen bertipe Ticket
tickets.iter_mut()    // Pinjam dengan boleh diubah, elemen bertipe &mut Ticket
```

Untuk kebanyakan kasus baca-baca data, `.iter()` adalah pilihan yang paling aman.

---

## map: Transformasi Tiap Elemen

`.map()` adalah "stasiun transformasi" di ban berjalan kita. Setiap elemen masuk, diubah jadi sesuatu yang lain, lalu keluar.

```rust
let ticket_ids: Vec<u32> = tickets.iter()
    .map(|t| t.id)
    .collect(); // collect() = kumpulkan hasilnya ke Vec
```

`|t| t.id` adalah closure yang mengambil sebuah ticket dan mengembalikan ID-nya saja.

---

## filter: Penyaringan

`.filter()` adalah "stasiun quality control". Hanya elemen yang lolos pengecekan yang diteruskan ke stasiun berikutnya. Closure di dalamnya harus mengembalikan `true` atau `false`.

```rust
let open_tickets: Vec<&Ticket> = tickets.iter()
    .filter(|t| t.status == "open")
    .collect();
```

Ticket dengan status selain "open" langsung dibuang di sini.

---

## collect: Kumpulkan Hasil

`.collect()` adalah "bak penampung" di ujung ban berjalan. Semua elemen yang sudah melewati berbagai tahap dikumpulkan menjadi satu koleksi, biasanya `Vec`.

Karena iterator bersifat lazy, **tidak ada yang terjadi** sampai kamu panggil `.collect()` (atau consumer lain). Baru di sinilah semua pekerjaan dilakukan sekaligus.

Kamu perlu bilang ke Rust tipe apa yang kamu mau, biasanya lewat anotasi tipe:

```rust
let hasil: Vec<String> = tickets.iter()
    .map(|t| t.title.clone())
    .collect();
```

---

## enumerate, count, sum, any, all, find

Selain `.collect()`, ada beberapa **consumer** (pemakan iterator) yang berguna:

**`.enumerate()`** menambahkan nomor urut ke tiap elemen. Berguna untuk menampilkan daftar bernomor.

```rust
for (i, ticket) in tickets.iter().enumerate() {
    println!("{}: [#{}] {}", i + 1, ticket.id, ticket.title);
}
// Hasilnya:
// 1: [#1] Login error
// 2: [#2] UI bug
// ...
```

**`.count()`** menghitung berapa elemen yang ada (setelah filter jika ada).

```rust
let open_count = tickets.iter()
    .filter(|t| t.status == "open")
    .count();
```

**`.sum()`** menjumlahkan semua elemen. Cocok untuk angka.

```rust
let total_priority: u8 = tickets.iter()
    .map(|t| t.priority)
    .sum();
```

**`.any()`** mengembalikan `true` jika *setidaknya satu* elemen memenuhi kondisi, dan berhenti segera saat ketemu yang cocok. **`.all()`** mengembalikan `true` hanya jika *semua* elemen memenuhi kondisi.

```rust
let ada_kritis = tickets.iter().any(|t| t.priority == 10);
let semua_selesai = tickets.iter().all(|t| t.status == "closed");
```

**`.find()`** mengembalikan elemen *pertama* yang cocok, dibungkus dalam `Option`. Kalau tidak ada yang cocok, kembalikan `None`.

```rust
let ticket_server = tickets.iter().find(|t| t.title.contains("Server"));
if let Some(t) = ticket_server {
    println!("Ketemu: {}", t.title);
}
```

**`.flat_map()`** adalah gabungan map + flatten. Berguna kalau closure menghasilkan koleksi dan kamu ingin hasilnya diratakan jadi satu koleksi besar. Misalnya, tiap ticket punya list tag, dan kamu mau kumpulkan semua tag dari semua ticket.

[ILUSTRASI: flat_map seperti membuka beberapa amplop, lalu menumpuk semua isinya menjadi satu tumpukan — bukan tumpukan amplop]

---

## Chaining: Gabungkan Semuanya

Kekuatan sesungguhnya muncul saat kamu **chain** (rantai) beberapa adapter. Ingat, semuanya lazy. Baru jalan saat consumer dipanggil.

```rust
#[derive(Debug, Clone)]
struct Ticket {
    id: u32,
    title: String,
    status: String,
    priority: u8,
}

fn main() {
    let tickets = vec![
        Ticket { id: 1, title: "Login error".to_string(), status: "open".to_string(), priority: 9 },
        Ticket { id: 2, title: "UI bug".to_string(), status: "closed".to_string(), priority: 3 },
        Ticket { id: 3, title: "Server down".to_string(), status: "open".to_string(), priority: 10 },
        Ticket { id: 4, title: "Lupa password".to_string(), status: "open".to_string(), priority: 5 },
    ];

    // Closure yang bisa dipakai ulang
    let is_urgent = |t: &Ticket| t.priority >= 8;

    // Chain: filter open → filter urgent → map ke string → collect
    let urgent_titles: Vec<String> = tickets.iter()
        .filter(|t| t.status == "open")
        .filter(|t| is_urgent(t))
        .map(|t| format!("[URGENT] {}", t.title))
        .collect();

    println!("Ticket urgent:");
    for title in &urgent_titles {
        println!("  - {}", title);
    }

    // Hitung ticket terbuka
    let open_count = tickets.iter()
        .filter(|t| t.status == "open")
        .count();
    println!("Ticket terbuka: {}", open_count);

    // Cek apakah ada yang kritis
    let ada_kritis = tickets.iter().any(|t| t.priority == 10);
    println!("Ada yang kritis: {}", ada_kritis);

    // Cari ticket spesifik
    let ticket_server = tickets.iter().find(|t| t.title.contains("Server"));
    if let Some(t) = ticket_server {
        println!("Ketemu: {}", t.title);
    }

    // Tampilkan dengan nomor urut
    for (i, ticket) in tickets.iter().enumerate() {
        println!("{}: [#{}] {}", i + 1, ticket.id, ticket.title);
    }
}
```

Output yang kamu harap:
```
Ticket urgent:
  - [URGENT] Login error
  - [URGENT] Server down
Ticket terbuka: 3
Ada yang kritis: true
Ketemu: Server down
1: [#1] Login error
2: [#2] UI bug
3: [#3] Server down
4: [#4] Lupa password
```

Bandingkan dengan cara manual pakai loop `for` + `if` + `push`. Iterator jauh lebih ringkas dan mudah dibaca.

---

## Latihan

**Latihan 1**: Dari daftar `tickets` di atas, buat `Vec<String>` berisi judul semua ticket dengan priority di atas 4, diawali dengan priority-nya dalam kurung siku. Contoh output: `"[9] Login error"`.

**Latihan 2**: Hitung rata-rata priority dari semua ticket yang statusnya `"open"`. Tips: gunakan `.map()` untuk ambil priority, `.sum()` untuk jumlahkan, `.count()` untuk hitung, lalu bagi.

**Latihan 3**: Cek apakah semua ticket dengan priority >= 9 berstatus `"open"`. Gunakan `.filter()` diikuti `.all()`.

**Latihan 4 (tantangan)**: Gunakan `.enumerate()` dan `.filter()` untuk menampilkan *posisi* (index 0-based) dari ticket yang berstatus `"closed"` di dalam slice asli.

---

Bab berikutnya masuk ke **error handling** yang lebih idiomatis di Rust. Bagaimana `Result` dan `?` operator membuat penanganan error bersih tanpa `unwrap()` bertebaran.
