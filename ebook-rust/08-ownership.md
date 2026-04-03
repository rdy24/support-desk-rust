# Bab 08: Ownership

Bayangkan kamu punya rumah. Rumah itu punya satu sertifikat, dan sertifikat itu hanya boleh dipegang oleh satu orang di satu waktu. Kalau kamu kasih sertifikatnya ke orang lain, kamu tidak bisa lagi klaim rumah itu. Begitulah cara Rust mengelola memori: setiap data punya satu pemilik, dan kepemilikan itu bisa berpindah tangan, tapi tidak bisa dipunyai bersama begitu saja.

Konsep ini namanya **ownership** (kepemilikan), dan ini adalah jantung dari Rust. Pahami ini, dan setengah perjalananmu belajar Rust sudah terbuka.

[ILUSTRASI: Satu rumah dengan satu sertifikat — hanya satu orang yang bisa pegang]

---

## Kenapa Rust Butuh Ownership?

Setiap program butuh memori untuk menyimpan data, seperti nama user, isi ticket, token login, dan sebagainya. Masalahnya: memori itu harus dibersihkan setelah tidak dipakai. Kalau tidak, programmu akan terus makan RAM sampai crash.

Ada dua cara yang umum dipakai bahasa lain. **Manual memory management** seperti di C dan C++: programmer sendiri yang tulis `free()` atau `delete`. Kalau lupa? Memory leak. Kalau salah waktu? Program crash dengan cara yang menyeramkan. Di sisi lain, **Garbage Collector** seperti di Java, Go, dan Python: ada "petugas kebersihan" otomatis yang jalan di background. Nyaman, tapi ada biaya, yaitu performa unpredictable dan program bisa pause tiba-tiba saat GC bekerja.

Rust pilih jalur ketiga: **ownership**. Aturannya sederhana, dicek waktu kompilasi (bukan saat program jalan), dan tidak butuh GC. Gratis di runtime, aman di compile time.

---

## 3 Aturan Ownership

Rust punya tiga aturan dasar yang tidak bisa dilanggar:

**1. Setiap nilai punya tepat satu owner (pemilik).**

```rust
let title = String::from("Login error"); // `title` adalah owner dari String ini
```

**2. Hanya boleh ada satu owner dalam satu waktu.**

Tidak ada dua variabel yang bisa "memiliki" data yang sama secara bersamaan.

**3. Ketika owner keluar scope, nilai di-drop: memori dibebaskan otomatis.**

```rust
{
    let title = String::from("Login error");
    // title valid di sini
} // <-- scope habis, title di-drop, memori dibebaskan
```

Scope artinya blok kode antara `{` dan `}`. Begitu keluar blok, owner hilang, data ikut hilang.

---

## Move Semantics

Ini bagian yang sering bikin pemula kaget. Bayangkan kamu punya sebuah buku fisik. Kalau kamu **kasih** buku itu ke temanmu, kamu tidak punya buku itu lagi, kamu tidak bisa baca, tidak bisa pinjamkan ke orang lain. Buku berpindah tangan.

Begitu juga di Rust. Saat kamu assign sebuah `String` ke variabel lain atau kirim ke fungsi, yang terjadi bukan duplikasi, melainkan **pemindahan kepemilikan** (move).

```rust
fn take_ownership(ticket_title: String) {
    println!("Memproses: {}", ticket_title);
    // ticket_title di-drop di sini
}

fn give_back(ticket_title: String) -> String {
    println!("Memproses: {}", ticket_title);
    ticket_title // dikembalikan ke pemanggil
}

fn main() {
    // Move semantics
    let title = String::from("Login error");
    take_ownership(title);
    // println!("{}", title); // ERROR! title sudah berpindah

    // Give back ownership
    let title2 = String::from("Server down");
    let title2 = give_back(title2); // ownership kembali
    println!("Masih bisa dipakai: {}", title2);

    // Copy types tidak move
    let id: u32 = 42;
    let id2 = id; // copy, bukan move
    println!("id: {}, id2: {}", id, id2); // keduanya valid

    // Clone untuk deep copy
    let title3 = String::from("Database error");
    let title3_copy = title3.clone();
    println!("Original: {}, Copy: {}", title3, title3_copy);
}
```

Perhatikan `take_ownership(title)`, setelah baris itu, `title` tidak bisa dipakai lagi. Kalau kamu coba, compiler langsung marah dengan pesan yang jelas. Ini bukan bug, ini fitur: Rust mencegahmu pakai data yang sudah "diberikan". Fungsi `give_back` menunjukkan cara mengembalikan ownership, cukup return nilainya, dan pemanggil mendapat ownership kembali.

---

## Copy Types: Tipe yang Tidak Move

Tapi tunggu, di contoh tadi, `id` dan `id2` keduanya masih valid setelah assignment. Kenapa?

Tipe seperti `u32`, `i32`, `bool`, `f64`, `char` adalah **Copy types**. Data mereka kecil dan tinggal di stack (bukan heap), jadi Rust cukup buat salinan otomatis. Tidak ada "move", yang terjadi adalah duplikasi murni.

```rust
let user_id: u32 = 101;
let backup_id = user_id; // copy terjadi di sini

println!("user_id: {}, backup_id: {}", user_id, backup_id); // keduanya valid
```

Tipe kompleks seperti `String` atau `Vec<T>` tidak bisa di-copy otomatis karena data mereka ada di heap dan ukurannya tidak tetap. Itulah kenapa Rust pakai move untuk mereka.

[ILUSTRASI: Angka di papan tulis bisa disalin, tapi dokumen asli hanya bisa dipegang satu orang]

---

## Clone: Duplikasi Eksplisit

Kadang kamu memang mau punya dua salinan data yang sama. Misalnya, kamu mau proses sebuah judul ticket tapi juga simpan salinannya untuk log. Untuk itu, pakai `.clone()`, yang membuat deep copy (salinan lengkap, termasuk data di heap).

```rust
let ticket_title = String::from("Database error");
let title_for_log = ticket_title.clone(); // salinan baru dibuat

println!("Proses: {}", ticket_title);
println!("Log: {}", title_for_log);
```

Jangan asal pakai `.clone()` di mana-mana. Clone itu ada biayanya, yaitu alokasi memori baru. Di bab selanjutnya kita akan belajar cara yang lebih efisien: **borrowing** (meminjam). Untuk sekarang, clone sudah cukup.

---

## Drop: Pembersihan Otomatis

Setiap kali sebuah owner keluar scope, Rust otomatis memanggil fungsi `drop` untuk membebaskan memori. Kamu tidak perlu tulis apapun.

```rust
fn create_ticket() {
    let title = String::from("Tidak bisa login");
    let description = String::from("User mendapat error 401");
    println!("Ticket: {} - {}", title, description);
} // <-- title dan description di-drop di sini, memori bersih

fn main() {
    create_ticket();
    // tidak ada memory leak
}
```

Urutan drop mengikuti urutan kebalikan deklarasi, variabel yang dideklarasi terakhir di-drop duluan. Ini penting kalau ada ketergantungan antar data, tapi intinya: **Rust bersihin sendiri, kamu tidak perlu mikir**.

---

## Kenapa Ini Lebih Baik dari GC dan Manual Memory?

| | Manual (C/C++) | Garbage Collector | Ownership (Rust) |
|---|---|---|---|
| Performa | Tinggi | Unpredictable | Tinggi dan konsisten |
| Keamanan | Rawan bug | Lebih aman | Dijamin aman |
| Runtime overhead | Tidak ada | Ada (GC pause) | Tidak ada |
| Waktu belajar | Susah | Mudah | Sedang |

Dengan ownership, Rust bisa memberikan jaminan keamanan memori **tanpa GC** dan **tanpa overhead runtime**. Semua pengecekan terjadi saat kompilasi. Kalau kode kamu berhasil dikompilasi, Rust sudah menjamin tidak ada memory leak, tidak ada use-after-free, tidak ada data race.

Untuk aplikasi seperti support desk kita, ini artinya kita bisa yakin bahwa data user dan ticket dikelola dengan benar, tanpa perlu khawatir tentang kebocoran memori saat traffic tinggi.

[ILUSTRASI: Diagram perbandingan — GC pause vs ownership compile-time check]

---

## Latihan

**1. Temukan errornya**

Kode di bawah tidak akan bisa dikompilasi. Kenapa? Bagaimana cara memperbaikinya?

```rust
fn print_status(status: String) {
    println!("Status ticket: {}", status);
}

fn main() {
    let status = String::from("open");
    print_status(status);
    print_status(status); // baris ini bermasalah
}
```

**2. Tulis fungsinya**

Buat fungsi `duplicate_title` yang menerima `String`, mencetak judulnya, dan mengembalikan dua salinan judul tersebut sebagai tuple `(String, String)`. Petunjuk: gunakan `.clone()`.

**3. Copy atau Move?**

Untuk masing-masing tipe berikut, tentukan apakah assignment akan menghasilkan copy atau move:
- `let a: i32 = 5; let b = a;`
- `let a = String::from("hello"); let b = a;`
- `let a: bool = true; let b = a;`
- `let a: Vec<u32> = vec![1, 2, 3]; let b = a;`

---

Di bab berikutnya, kita akan belajar cara "meminjam" data tanpa harus memindahkan kepemilikannya, konsep yang disebut **borrowing** dan **references**. Ini yang akan membuat kode Rust kamu jauh lebih fleksibel tanpa mengorbankan keamanan.
