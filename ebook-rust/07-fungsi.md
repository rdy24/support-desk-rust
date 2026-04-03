# Bab 07: Fungsi

Sejauh ini semua kode Rust kita berjejer di satu tempat. Bayangkan dapur yang cuma punya satu meja panjang, semua bahan, alat masak, dan proses memasak bercampur jadi satu. Kacau.

Fungsi adalah cara kita **memecah kode jadi bagian-bagian kecil yang punya tugas masing-masing**, seperti memisahkan stasiun kerja di dapur: satu meja untuk potong sayur, satu untuk masak, satu untuk plating. Lebih rapi, lebih mudah diulang, dan kalau ada yang salah, kita tahu persis di mana harus cari.

[ILUSTRASI: Dapur restoran dengan beberapa stasiun kerja terpisah — masing-masing punya label dan tugas sendiri, dihubungkan oleh alur kerja yang jelas]

---

## Syntax Dasar

Di Rust, fungsi ditulis dengan kata kunci `fn` (singkatan dari *function*). Formatnya seperti ini:

```rust
fn nama_fungsi(parameter: Tipe) -> TipeReturn {
    // isi fungsi
}
```

Contoh paling sederhana:

```rust
fn greet_user(name: &str) -> String {
    format!("Halo, {}! Selamat datang di Support Desk.", name)
}
```

`fn` adalah kata kunci untuk mendefinisikan fungsi. `greet_user` adalah namanya, ditulis dalam **snake_case** (huruf kecil, kata dipisah underscore). `name: &str` adalah parameter bertipe string slice, dan `-> String` menandakan fungsi ini mengembalikan `String`.

### Snake Case: Konvensi Penamaan

Di Rust, nama fungsi **wajib** pakai snake_case. Bukan `sapa_User`, bukan `sapaUser`, tapi `sapa_user`. Ini bukan sekadar gaya. Rust compiler akan kasih peringatan kalau kamu pakai konvensi lain. Anggap ini sebagai "aturan tidak tertulis yang sudah jadi tertulis."

---

## Parameter

Parameter adalah "bahan masukan" yang kita kasih ke fungsi. Setiap parameter harus punya **nama** dan **tipe**, karena Rust tidak mau tebak-tebakan.

```rust
fn format_ticket(id: u32, title: &str, priority: u8) -> String {
    format!("[#{}] {} (prioritas: {})", id, title, priority)
}
```

Tiga parameter di sini: `id: u32` untuk nomor ticket (bilangan bulat tak bertanda 32-bit), `title: &str` untuk judul ticket berupa referensi ke string, dan `priority: u8` untuk angka prioritas 0–255 (8-bit, cukup untuk skala 1–10). Kalau ada lebih dari satu parameter, pisahkan dengan koma, dan setiap parameter harus punya tipe sendiri.

---

## Return Value: Expression Terakhir

Ini bagian yang sering bikin pemula bengong: **di Rust, nilai yang dikembalikan fungsi adalah expression terakhir di dalam fungsi, tanpa titik koma.**

Coba bandingkan:

```rust
fn calculate_priority(score: u8) -> u8 {
    score * 2   // ← tidak ada titik koma — ini return value-nya!
}
```

Kalau kamu tambahkan titik koma:

```rust
fn calculate_priority(score: u8) -> u8 {
    score * 2;  // ← sekarang ini jadi statement, bukan return value!
    // Rust akan error karena fungsi dijanjikan return u8, tapi tidak ada
}
```

Aturan sederhananya: **expression terakhir tanpa titik koma = nilai yang dikembalikan**. Ini beda dari banyak bahasa lain yang pakai kata kunci `return` untuk semua kasus.

---

## `return` Eksplisit: Early Return

Meskipun Rust punya cara elegan di atas, kadang kita perlu keluar dari fungsi lebih awal, misalnya kalau kondisi tertentu terpenuhi. Bayangkan validasi ticket: kalau ID-nya nol, kita langsung tolak tanpa proses lebih lanjut.

```rust
fn validate_ticket_id(id: u32) -> bool {
    if id == 0 {
        return false;  // early return — langsung keluar
    }

    // lanjutkan proses validasi lain...
    id <= 999_999
}
```

`return false;` di sini adalah **early return**, keluar sebelum sampai ke akhir fungsi. Perhatikan pakai titik koma karena ini adalah statement `return`, bukan expression biasa. Pola ini berguna untuk menangani kasus "tidak valid" di awal sebelum logika utama berjalan. Kode jadi lebih mudah dibaca karena kasus-kasus khusus sudah diselesaikan duluan.

---

## Expression vs Statement

Ini konsep penting di Rust yang membedakannya dari banyak bahasa lain.

**Statement**: instruksi yang melakukan sesuatu, tapi **tidak menghasilkan nilai**.

```rust
let x = 5;          // statement — deklarasi variabel
println!("halo");   // statement — cetak ke layar
```

**Expression**: sesuatu yang **menghasilkan nilai**.

```rust
5 + 3               // expression — menghasilkan 8
"hello"             // expression — menghasilkan &str
if x > 0 { 1 } else { -1 }  // expression — menghasilkan 1 atau -1
```

Yang menarik di Rust: **blok `{ ... }` juga bisa jadi expression!**

```rust
let label = {
    let score = 7;
    if score >= 8 {
        "Kritis"
    } else {
        "Normal"
    }
    // expression terakhir tanpa titik koma = nilai blok
};
// label sekarang berisi "Normal"
```

[ILUSTRASI: Diagram dua kolom — kiri bertuliskan "Statement (tidak punya nilai, diakhiri titik koma)" dengan contoh-contohnya, kanan bertuliskan "Expression (punya nilai, bisa disimpan)" dengan contoh-contohnya]

Pemahaman ini penting karena Rust sangat konsisten: **titik koma mengubah expression menjadi statement** dan membuang nilai yang dihasilkan.

---

## Fungsi Tanpa Return Value

Tidak semua fungsi perlu mengembalikan sesuatu. Fungsi yang hanya mencetak ke layar atau menutup ticket tugasnya adalah *melakukan aksi*, bukan *menghasilkan nilai*.

```rust
fn close_ticket(id: u32) {
    println!("Ticket #{} ditutup.", id);
    // return () secara implisit
}
```

Ketika fungsi tidak punya `-> Tipe`, Rust secara otomatis menganggap return type-nya adalah `()`, dibaca "unit". Ini tipe kosong yang artinya "tidak ada nilai berarti yang dikembalikan." Kamu bisa tulis eksplisit kalau mau:

```rust
fn close_ticket(id: u32) -> () {
    println!("Ticket #{} ditutup.", id);
}
```

Tapi biasanya kita tidak perlu repot, cukup skip bagian `->` dan Rust paham sendiri.

---

## Multiple Return dengan Tuple

Bagaimana kalau kita ingin kembalikan lebih dari satu nilai sekaligus? Jawabannya adalah **tuple**, paket nilai yang dibungkus dalam tanda kurung.

```rust
fn categorize_priority(score: u8) -> (&'static str, bool) {
    if score >= 8 {
        ("Kritis", true)
    } else if score >= 5 {
        ("Sedang", false)
    } else {
        ("Rendah", false)
    }
}
```

Return type-nya `(&'static str, bool)`, tuple berisi dua nilai: label prioritas dan status urgent. `&'static str` artinya string yang "hidup seumur program", cocok untuk string literal yang kita tulis langsung di kode. Lifetime dibahas lebih dalam di bab-bab selanjutnya.

Untuk menggunakan hasilnya, kita bisa *destructure* tuple:

```rust
let (label, urgent) = categorize_priority(9);
println!("Prioritas: {}, Urgent: {}", label, urgent);
// Output: Prioritas: Kritis, Urgent: true
```

---

## Contoh Lengkap

```rust
fn format_ticket(id: u32, title: &str, priority: u8) -> String {
    format!("[#{}] {} (prioritas: {})", id, title, priority)
}

fn categorize_priority(score: u8) -> (&'static str, bool) {
    if score >= 8 {
        ("Kritis", true)
    } else if score >= 5 {
        ("Sedang", false)
    } else {
        ("Rendah", false)
    }
}

fn close_ticket(id: u32) {
    println!("Ticket #{} ditutup.", id);
}

fn main() {
    let formatted = format_ticket(1, "Login error", 9);
    println!("{}", formatted);

    let (label, urgent) = categorize_priority(9);
    println!("Prioritas: {}, Urgent: {}", label, urgent);

    close_ticket(1);
}
```

Output yang diharapkan:
```
[#1] Login error (prioritas: 9)
Prioritas: Kritis, Urgent: true
Ticket #1 ditutup.
```

---

## Latihan

Coba kerjakan ini sebelum lanjut ke bab berikutnya:

1. **Tulis fungsi `greet_user`** yang menerima `username: &str` dan `role: &str`, lalu mengembalikan `String` berisi pesan sambutan. Contoh output: `"Selamat datang, budi! Kamu login sebagai admin."`

2. **Tulis fungsi `ticket_summary`** yang menerima `id: u32`, `status: &str`, dan `days_open: u32`, lalu mengembalikan tuple `(String, bool)`: String berisi ringkasan ticket, bool berisi apakah ticket sudah lebih dari 7 hari terbuka.

3. **Tantangan:** Tulis fungsi `assign_ticket` yang menerima `ticket_id: u32` dan `agent_name: &str`. Kalau `agent_name` kosong (string `""`), lakukan early return dengan mencetak pesan error. Kalau tidak, cetak pesan assignment.

Coba tanpa melihat kode di atas dulu, kalau stuck, baru scroll ke atas.
