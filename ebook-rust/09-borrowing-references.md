# Bab 09: Borrowing dan References

Di bab sebelumnya kita belajar tentang ownership, setiap nilai punya satu pemilik, dan kalau dipindah ke fungsi lain, nilai itu "berpindah tangan". Masalahnya, kalau kita harus terus-terusan mengembalikan nilai ke pemanggil, kode jadi ribet. Di sinilah *borrowing* dan *references* masuk sebagai solusi.

---

## Masalah Tanpa Borrowing

Bayangkan kita punya fungsi untuk mengecek panjang judul tiket. Tanpa borrowing, kita harus *move* String ke fungsi, lalu mengembalikannya lagi supaya bisa dipakai setelah pemanggilan:

```rust
fn title_length(title: String) -> (String, usize) {
    let length = title.len();
    (title, length) // harus kembalikan title supaya tidak hilang
}

fn main() {
    let title = String::from("Login tidak bisa");
    let (title, length) = title_length(title);
    println!("Judul: '{}', panjang: {} karakter", title, length);
}
```

Ini berantakan. Fungsi yang harusnya cuma "mengecek" malah harus ikut urusan kepemilikan. Solusinya: *references*.

[ILUSTRASI: Diagram dua jalur — kiri tanpa reference (nilai berpindah tangan bolak-balik), kanan dengan reference (nilai tetap di tempat, fungsi cuma "melirik")]

---

## Reference: Meminjam Tanpa Mengambil

**Reference** adalah cara kita memberi akses ke sebuah nilai *tanpa* memindahkan kepemilikannya. Simbol yang dipakai adalah `&` (dibaca "ampersand" atau cukup "ref").

Analoginya seperti baca buku di perpustakaan. Kamu tidak bawa pulang bukunya, kamu baca di tempat, selesai, buku kembali ke rak. Pemilik buku (perpustakaan) tetap pegang bukunya. Inilah yang disebut *meminjam* (*borrowing*). Dalam Rust, `&` artinya: "aku mau lihat nilai ini, tapi aku tidak mau ambil alih kepemilikannya."

```rust
fn title_length(title: &str) -> usize {
    title.len()
    // title dikembalikan otomatis, tidak ada move
}

fn main() {
    let title = String::from("Login tidak bisa");
    let length = title_length(&title); // kita pinjamkan &title
    println!("Judul: '{}', panjang: {} karakter", title, length);
    // title masih valid karena kita hanya meminjam
}
```

Jauh lebih bersih. Fungsi `title_length` cukup terima `&str` (reference ke string), pakai, lalu selesai. Tidak ada bolak-balik ownership.

---

## Immutable Reference (`&T`)

Reference biasa (`&T`) bersifat *immutable*, kita hanya bisa **membaca** nilai tersebut, tidak bisa mengubahnya. Setara dengan akses *read-only*.

Nah, di sinilah ada kelonggaran menarik: **kita boleh punya banyak immutable reference sekaligus** ke nilai yang sama. Tidak ada masalah kalau sepuluh fungsi sekaligus ingin "membaca" tiket yang sama.

```rust
fn main() {
    let t = String::from("Bug database");
    let r1 = &t;
    let r2 = &t;
    println!("{} dan {}", r1, r2); // OK! Dua reference sekaligus
}
```

Logikanya sederhana: kalau semua orang hanya membaca, tidak ada yang bisa merusak data. Aman.

---

## Mutable Reference (`&mut T`)

Kalau kita ingin *mengubah* nilai lewat reference, kita butuh **mutable reference** dengan simbol `&mut`. Ada aturan ketat di sini: **hanya boleh ada satu `&mut` dalam satu waktu**, dan tidak boleh ada `&` (immutable) lain yang masih aktif bersamaan.

```rust
fn add_prefix(title: &mut String, prefix: &str) {
    title.insert_str(0, prefix);
}

fn main() {
    let mut ticket_title = String::from("Server error");
    add_prefix(&mut ticket_title, "[KRITIS] ");
    println!("Setelah prefix: {}", ticket_title);
    // Output: [KRITIS] Server error
}
```

Perhatikan tiga hal: variabel `ticket_title` harus dideklarasikan dengan `mut`, saat memanggil fungsi kita tulis `&mut ticket_title`, dan di dalam fungsi parameternya bertipe `&mut String`.

---

## Aturan Borrowing

Rust punya dua aturan sederhana yang tidak bisa dilanggar:

**Aturan 1:** Boleh punya **banyak** immutable reference (`&T`) sekaligus.

**Aturan 2:** Atau, hanya boleh ada **satu** mutable reference (`&mut T`). Saat itu tidak boleh ada reference lain (termasuk immutable) yang masih aktif.

Singkatnya: **banyak pembaca** ATAU **satu penulis**. Tidak bisa keduanya bersamaan.

```rust
fn main() {
    let mut t = String::from("Bug database");

    // Beberapa immutable reference boleh
    let r1 = &t;
    let r2 = &t;
    println!("{} dan {}", r1, r2); // OK!

    // Setelah r1 dan r2 tidak dipakai lagi, baru boleh mutable reference
    let r3 = &mut t;
    r3.push_str(" - sedang diperiksa");
    println!("{}", r3); // OK!

    // Ini yang TIDAK boleh:
    // let r4 = &t;      // immutable
    // let r5 = &mut t;  // ERROR! r4 masih aktif
}
```

Itulah kenapa aturan ini ada. Bayangkan dua orang mengedit dokumen yang sama secara bersamaan tanpa koordinasi, hasilnya kacau. Rust mencegah situasi ini di level bahasa.

[ILUSTRASI: Papan tulis besar. Skenario kiri: banyak orang berdiri dan membaca papan (boleh). Skenario kanan: satu orang memegang spidol dan menulis, semua orang lain mundur (hanya satu penulis)]

---

## Borrow Checker: Si Penjaga

Semua aturan borrowing di atas dijaga oleh komponen di dalam compiler Rust yang disebut **borrow checker**. Ia bekerja saat kamu menjalankan `cargo build` atau `cargo check`, bukan saat program dijalankan.

Artinya: kalau kode kamu melanggar aturan borrowing, program tidak akan berhasil dikompilasi. Kamu akan dapat pesan error yang jelas sebelum kode sampai ke production. Ini terasa ketat di awal, tapi inilah yang membuat Rust bisa menjamin tidak ada *data race*, yaitu kondisi di mana dua bagian kode saling bertabrakan saat mengakses data yang sama.

Pesan error dari borrow checker biasanya sangat deskriptif. Contoh:

```
error[E0502]: cannot borrow `t` as mutable because it is also borrowed as immutable
```

Baca pesannya, borrow checker sering langsung menunjuk baris mana yang bermasalah dan kenapa.

---

## Dangling Reference: Yang Tidak Mungkin Terjadi di Rust

**Dangling reference** (*reference menggantung*) adalah kondisi di mana sebuah reference menunjuk ke memori yang sudah tidak valid, misalnya karena nilai aslinya sudah dihapus atau keluar dari scope.

Di bahasa lain seperti C, ini bisa terjadi dan menyebabkan bug yang sulit dilacak. Rust menyelesaikan ini dengan mencegahnya saat compile time:

```rust
fn make_reference() -> &String { // ERROR!
    let s = String::from("tiket sementara");
    &s // s akan dihapus saat fungsi selesai — reference-nya jadi dangling
}
```

Rust akan menolak kode ini karena `s` habis masa hidupnya saat fungsi `make_reference` selesai, tapi kita mencoba mengembalikan reference ke `s`. Solusinya biasanya: kembalikan nilai langsung (move ownership), bukan reference-nya.

```rust
fn make_string() -> String {
    let s = String::from("tiket sementara");
    s // kembalikan nilai, bukan reference
}
```

---

## Sekilas tentang Lifetime

Bagaimana Rust tahu seberapa lama sebuah reference "boleh hidup"? Jawabannya adalah konsep yang disebut **lifetime**, cara Rust melacak "umur" dari setiap reference dan memastikan reference tidak hidup lebih lama dari nilai yang ia tunjuk.

Kabar baiknya: untuk banyak kasus, Rust bisa menebak lifetime secara otomatis (*lifetime elision*). Kamu tidak harus selalu menuliskannya. Lifetime akan dibahas lebih dalam di bab tersendiri. Untuk sekarang, cukup tahu bahwa borrow checker menggunakan lifetime di balik layar saat memvalidasi kode kita.

---

## Latihan

**Soal 1:** Buat fungsi `summarize_ticket` yang menerima reference ke `String` (judul tiket) dan mengembalikan `bool`, apakah panjang judulnya lebih dari 20 karakter. Pastikan string asli masih bisa dipakai setelah fungsi dipanggil.

**Soal 2:** Buat fungsi `close_ticket` yang menerima mutable reference ke `String` dan menambahkan teks `" [CLOSED]"` di akhir judul. Panggil fungsi itu, lalu cetak hasilnya.

**Soal 3 (tantangan):** Coba buat dua immutable reference ke sebuah String, cetak keduanya, lalu buat satu mutable reference dan ubah nilainya. Pastikan kamu tidak mencampur immutable dan mutable reference dalam scope yang sama.

Kalau dapat error dari borrow checker, baca pesannya, biasanya sudah cukup jelas menunjukkan masalahnya di mana.

---

Di bab berikutnya kita akan lihat bagaimana konsep ownership dan borrowing bekerja bersama *slice*, cara kita mereferensikan sebagian dari sebuah koleksi data.
