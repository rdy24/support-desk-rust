# Bab 05: String di Rust

## Kenapa Ada Dua Jenis String?

Di JavaScript atau Python, string itu simpel, tinggal pakai, selesai. Di Rust, ada dua jenis string: `&str` dan `String`. Wajar kalau bingung di awal.

Alasannya: Rust sangat peduli soal **siapa yang "punya" data** dan **di mana data itu disimpan di memori**. Bukan kerumitan tanpa alasan, justru inilah yang bikin program Rust cepat dan aman.

### Stack dan Heap: Dua Gudang Memori

Sebelum masuk ke dua jenis string, kamu perlu tahu dua tempat penyimpanan data di memori komputer:

**Stack** itu seperti tumpukan piring di kantin — cepat, rapi, tapi ukurannya terbatas dan harus ditumpuk sesuai urutan. Data yang ukurannya sudah diketahui saat kompilasi (angka, boolean, pointer) disimpan di sini. Menyimpan dan mengambil data dari stack sangat cepat.

**Heap** itu seperti gudang besar — lebih lambat untuk diakses, tapi bisa menyimpan data dengan ukuran yang baru diketahui saat program berjalan (misalnya teks yang diketik user). Data di heap diakses lewat pointer (alamat) yang disimpan di stack.

Kenapa ini penting? Karena di Rust, `&str` dan `String` berbeda justru karena **di mana datanya disimpan** dan **siapa yang bertanggung jawab** atas memori tersebut.

[ILUSTRASI: dua meja kerja — satu meja ada tumpukan fotokopi dokumen bertulisan "&str", meja lain ada satu dokumen asli bertulisan "String". Orang di meja fotokopi hanya bisa baca, orang di meja dokumen asli bisa coret-coret]

---

## `&str`: String Pinjaman

Bayangkan kamu kerja di kantor dan butuh lihat isi sebuah dokumen. Kamu minta fotokopinya. Bisa baca isinya, tapi tidak bisa mengedit dokumen aslinya, dan tidak perlu ngurusin penyimpanannya.

Itulah `&str` (dibaca: "string slice" atau "string ref"). Data-nya sudah ada di memori, langsung di program (binary) atau di tempat lain. Kamu hanya **meminjam** referensinya (`&` artinya borrow), tidak bisa mengubah isinya, dan tidak bertanggung jawab hapus atau bebaskan memorinya.

```rust
let username: &str = "budi";
```

Teks `"budi"` disimpan langsung di dalam file program kita (read-only section), tertanam di binary saat kompilasi. Variabel `username` hanya menyimpan "alamat" (pointer) dan panjang string itu, bukan menyimpan string itu sendiri di stack.

---

## `String`: String Milik Sendiri

Sekarang bayangkan kamu yang pegang **dokumen aslinya**. Bisa coret, tambah tulisan, atau sobek halamannya. Dan karena kamu yang pegang, kamu yang bertanggung jawab.

Itulah `String` (dengan huruf S kapital). Data-nya disimpan di **heap**, area memori yang fleksibel dan bisa bertumbuh. Kamu **memiliki** data ini (owned), bisa mengubah isinya, dan Rust akan otomatis membebaskan memori saat variabel ini tidak dipakai lagi.

```rust
let email = String::from("budi@example.com");
```

---

## Kapan Pakai Yang Mana?

Pakai `&str` kalau kamu menerima string sebagai parameter fungsi dan tidak perlu mengubahnya, kalau pakai string literal langsung di kode, atau kalau hanya perlu baca tanpa simpan jangka panjang.

Pakai `String` kalau kamu perlu **mengubah** isi string (tambah, hapus, modifikasi), mau **kembalikan string dari fungsi**, atau mau **simpan string di dalam struct**.

Contoh nyata: fungsi `greet_user` cukup terima `&str` karena hanya butuh baca nama. Hasilnya adalah `String` yang baru dibuat di dalam fungsi.

```rust
fn greet_user(name: &str) -> String {
    format!("Halo, {}! Selamat datang di Support Desk.", name)
}
```

---

## Konversi Bolak-Balik

**`&str` → `String`:**

```rust
let name_ref: &str = "budi";
let name_owned: String = name_ref.to_string();
// atau
let name_owned2: String = String::from("budi");
```

**`String` → `&str`:**

```rust
let email = String::from("budi@example.com");
let email_ref: &str = &email;
// atau
let email_ref2: &str = email.as_str();
```

Perhatikan `&` di depan `email`, itu tanda "pinjam". Kepemilikan tidak berpindah, hanya membuat referensi sementara.

[ILUSTRASI: diagram panah dua arah antara kotak "&str" dan kotak "String", dengan label ".to_string()" ke kanan dan "&" ke kiri]

---

## Membuat String Baru: `format!`

Macro `format!` adalah cara paling nyaman untuk menggabungkan teks. Hasilnya selalu `String`.

```rust
let username = "budi";
let ticket_id = 42;

let pesan = format!("Tiket #{} dibuat oleh {}", ticket_id, username);
// Hasil: "Tiket #42 dibuat oleh budi"
```

`{}` adalah **placeholder**: tempat nilai variabel dimasukkan.

Mungkin kamu tergoda pakai `+` seperti di JavaScript:

```rust
let title = String::from("Masalah Login");
let suffix = " - Urgent";

let new_title = title + suffix; // title sudah TIDAK bisa dipakai lagi!
```

Nah, di sinilah ada hal unik: setelah operasi `+`, variabel `title` **berpindah kepemilikan** ke `new_title`. Kamu tidak bisa pakai `title` lagi. Gunakan `format!` saja, lebih jelas dan tidak ada efek samping ownership:

```rust
let title = String::from("Masalah Login");
let suffix = " - Urgent";

let new_title = format!("{}{}", title, suffix);
// title masih bisa dipakai di sini!
```

---

## Method-Method Berguna

Baik `&str` maupun `String` punya banyak method. Beberapa yang sering dipakai:

```rust
let email = "  Budi@Example.COM  ";

// Panjang string dalam bytes
println!("{}", email.len());          // 20

// Cek apakah mengandung substring tertentu
println!("{}", email.contains('@'));   // true

// Ubah semua jadi huruf kapital
println!("{}", email.to_uppercase()); // "  BUDI@EXAMPLE.COM  "

// Hapus spasi di awal dan akhir
println!("{}", email.trim());         // "Budi@Example.COM"
```

`.trim()` berguna untuk bersihin input dari user yang tidak sengaja pencet spasi. `.contains('@')` untuk validasi format email.

---

## Contoh: Nama dan Email User

```rust
fn greet_user(name: &str) -> String {
    format!("Halo, {}! Selamat datang di Support Desk.", name)
}

fn main() {
    let username: &str = "budi";                    // &str literal
    let email = String::from("budi@example.com");   // String owned

    let greeting = greet_user(username);
    println!("{}", greeting);
    println!("Email: {}", email);

    // Method berguna
    println!("Panjang username: {}", username.len());
    println!("Mengandung '@': {}", email.contains('@'));

    // Konversi
    let username_owned: String = username.to_string();
    let email_ref: &str = &email;
    println!("Owned: {}, Ref: {}", username_owned, email_ref);
}
```

Output-nya:
```
Halo, budi! Selamat datang di Support Desk.
Email: budi@example.com
Panjang username: 4
Mengandung '@': true
Owned: budi, Ref: budi@example.com
```

Fungsi `greet_user` menerima `&str` (pinjam saja) tapi mengembalikan `String` karena string baru dibuat di dalam fungsi dan perlu dikirim keluar.

---

## Latihan

1. **Buat fungsi `format_ticket_title`** yang menerima `category: &str` dan `id: u32`, lalu mengembalikan `String` dengan format `"[KATEGORI] Tiket #ID"`. Contoh hasil: `"[LOGIN] Tiket #7"`. Gunakan `format!` dan `.to_uppercase()`.

2. **Validasi email sederhana**: buat variabel `email` bertipe `String`, lalu cek apakah mengandung `'@'` dan panjangnya lebih dari 5 karakter. Cetak `"Email valid"` atau `"Email tidak valid"`.

3. **Konversi bolak-balik**: buat `&str` dari string literal `"support@desk.com"`, konversi ke `String` dengan `.to_string()`, lalu konversi kembali ke `&str` dengan `&`. Cetak keduanya.
