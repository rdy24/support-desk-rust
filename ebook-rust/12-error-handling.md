# Bab 12: Error Handling

Semua program pasti bisa gagal. File tidak ditemukan, koneksi database putus, input user aneh: itu hal biasa. Yang membedakan program bagus dan program buruk bukan "apakah error terjadi", tapi "bagaimana program merespons error itu".

---

## Analogi: Kasir yang Jujur vs Kasir yang Diam dan Meledak

Bayangkan kamu pesan burger di restoran cepat saji.

**Kasir A (jujur):** "Maaf Kak, stok patty untuk menu itu habis. Mau pilih menu lain atau refund?"

**Kasir B (bermasalah):** Diam, manggut-manggut, kamu bayar, duduk, tunggu 30 menit... terus tiba-tiba meja kamu meledak.

Kasir A merepresentasikan program yang mengakui ada masalah dan memberi pilihan. Kasir B adalah program yang crash tiba-tiba tanpa penjelasan. Rust memaksa kamu jadi Kasir A.

---

## Dua Jenis Error di Rust

Rust membagi error jadi dua kategori. **Recoverable** adalah error yang wajar terjadi dan bisa direspons, contohnya ticket tidak ditemukan, user salah input, atau file belum ada. Di Rust, ini direpresentasikan dengan tipe `Result<T, E>`. **Unrecoverable** adalah bug serius yang *seharusnya tidak pernah terjadi* dalam program yang benar, contohnya akses index array di luar batas. Di Rust, ini memicu `panic!` dan program langsung berhenti.

Perbedaannya penting: jangan pakai `panic!` untuk kondisi normal seperti "user salah input password". Itu bukan bug, itu flow yang memang mungkin terjadi.

---

## `Result<T, E>`: Error yang Bisa Ditangani

`Result<T, E>` adalah *enum* (tipe data yang punya beberapa kemungkinan nilai) dengan dua varian:

```rust
enum Result<T, E> {
    Ok(T),   // sukses, berisi nilai bertipe T
    Err(E),  // gagal, berisi error bertipe E
}
```

`T` adalah tipe data kalau sukses (misalnya `String`, `User`, `Ticket`), dan `E` adalah tipe error kalau gagal. Jadi fungsi yang bisa gagal tidak langsung *return* nilainya, melainkan *return* `Result` yang membungkus nilai atau error.

[ILUSTRASI: diagram kotak berlabel "Result" yang di dalamnya ada dua jalur: jalur hijau "Ok(nilai)" dan jalur merah "Err(error)"]

---

## Menangani Result dengan `match`

`match` adalah cara paling eksplisit untuk menangani `Result`:

```rust
fn find_ticket(id: u32) -> Result<String, String> {
    if id == 0 {
        return Err("Ticket tidak ditemukan".to_string());
    }
    Ok(format!("Ticket #{}: Login error", id))
}

fn main() {
    match find_ticket(1) {
        Ok(ticket) => println!("Berhasil: {}", ticket),
        Err(e) => println!("Gagal: {}", e),
    }
}
```

`match` memaksa kamu menangani *kedua kemungkinan*: sukses dan gagal. Kalau kamu skip salah satu, compiler akan komplain. Kamu tidak bisa pura-pura error tidak ada.

---

## Operator `?`: Propagasi Elegan

Kadang fungsi kita memanggil fungsi lain yang bisa gagal, dan kalau gagal, kita mau *langsung teruskan error itu ke atas* (ke pemanggil fungsi kita). Ini disebut *error propagation*.

Tanpa `?`, kode kita jadi bertele-tele:

```rust
fn get_ticket(id: u32) -> Result<String, String> {
    let result = find_ticket(id);
    match result {
        Ok(t) => Ok(t),
        Err(e) => return Err(e), // teruskan error ke atas
    }
}
```

Dengan operator `?`, kode yang sama jadi:

```rust
fn get_ticket(id: u32) -> Result<String, String> {
    let ticket = find_ticket(id)?; // kalau Err, langsung return Err
    Ok(ticket)
}
```

Tanda `?` artinya: "kalau ini `Ok`, ambil nilainya dan lanjut. Kalau `Err`, langsung kembalikan error itu dari fungsi ini."

> **Penting:** `?` hanya bisa dipakai di fungsi yang *return* `Result` atau `Option`. Kalau dipakai di `main()` biasa atau fungsi yang return `()`, compiler akan error.

---

## `unwrap()` dan `expect()`

Dua method ini adalah cara *paksa* membuka `Result`:

```rust
let ticket = find_ticket(1).unwrap(); // ambil nilai, atau panic kalau Err
let ticket = find_ticket(1).expect("Gagal ambil ticket"); // sama, tapi pesan custom
```

Keduanya **berbahaya** kalau dipakai sembarangan: kalau ternyata hasilnya `Err`, program langsung panic dan crash. Boleh dipakai di kode prototipe atau eksperimen ketika kamu tahu pasti nilainya `Ok` dan tidak mau ribet dulu, atau di dalam test, atau kalau ada logika di atasnya yang sudah memastikan nilai pasti `Ok`. Jangan pakai di kode produksi untuk kondisi yang memang mungkin gagal secara normal seperti input dari user atau query database.

`expect()` lebih baik dari `unwrap()` karena pesan error-nya lebih informatif saat debugging.

---

## Custom Error dengan `thiserror`

Di contoh sebelumnya kita pakai `String` sebagai tipe error. Ini simpel tapi kurang ideal karena kita kehilangan informasi tentang *jenis* error apa yang terjadi.

Solusinya: buat tipe error sendiri menggunakan crate `thiserror`.

> **Crate** = library di ekosistem Rust (seperti package di Node.js atau pip di Python).

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
thiserror = "2"  # saat ebook ini ditulis, Maret 2026
```

Lalu definisikan error:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error("Ticket tidak ditemukan: id={0}")]
    TicketNotFound(u32),
    #[error("Akses ditolak: {0}")]
    Unauthorized(String),
    #[error("Input tidak valid: {0}")]
    ValidationError(String),
}
```

`#[derive(Debug, Error)]` adalah macro yang otomatis mengimplementasi trait `Debug` dan `Error` untuk enum kita. Trait itu seperti kontrak kemampuan: "enum ini bisa di-debug dan diperlakukan sebagai error". `#[error("...")]` mendefinisikan pesan error yang tampil saat di-print, dan `{0}` adalah placeholder untuk nilai pertama di dalam varian enum itu.

Sekarang fungsi-fungsi kita bisa pakai `AppError`:

```rust
fn find_ticket(id: u32) -> Result<String, AppError> {
    if id == 0 {
        return Err(AppError::TicketNotFound(id));
    }
    Ok(format!("Ticket #{}: Login error", id))
}

fn get_ticket_for_user(ticket_id: u32, user_role: &str) -> Result<String, AppError> {
    if user_role == "banned" {
        return Err(AppError::Unauthorized("User diblokir".to_string()));
    }
    let ticket = find_ticket(ticket_id)?; // ? meneruskan AppError ke atas
    Ok(ticket)
}

fn main() {
    match get_ticket_for_user(1, "customer") {
        Ok(ticket) => println!("Berhasil: {}", ticket),
        Err(e) => println!("Error: {}", e),
    }

    match get_ticket_for_user(0, "customer") {
        Ok(ticket) => println!("Berhasil: {}", ticket),
        Err(e) => println!("Error: {}", e),
    }
}
```

Output:

```
Berhasil: Ticket #1: Login error
Error: Ticket tidak ditemukan: id=0
```

Nah, di sinilah custom error jadi berguna: kode yang memanggil fungsi kita bisa melakukan `match` berdasarkan jenis error, misalnya `TicketNotFound` return HTTP 404, `Unauthorized` return HTTP 403.

[ILUSTRASI: diagram alur dari `get_ticket_for_user` — jika user "banned" panah ke `Unauthorized`, jika id=0 panah ke `TicketNotFound`, jika sukses panah ke `Ok(ticket)`]

---

## `panic!`: Untuk Bug yang Seharusnya Tidak Terjadi

`panic!` menghentikan program secara paksa. Gunakan ini hanya untuk kondisi yang *seharusnya tidak pernah terjadi* dalam program yang benar, bukan untuk kondisi normal yang bisa terjadi dari input user.

```rust
fn set_ticket_priority(level: u8) {
    if level > 5 {
        panic!("Priority level tidak valid: {}. Harusnya 1-5.", level);
    }
    // ...
}
```

Konteks yang tepat untuk `panic!`: kondisi yang menandakan *bug di kode kamu sendiri* (bukan bug dari input user), invariant yang *dijamin* oleh logika program tidak akan pernah rusak tapi kalau rusak berarti program sudah dalam keadaan korup, atau saat setup awal gagal (misalnya tidak bisa membaca config file yang harus selalu ada). Untuk semua kondisi yang bisa terjadi secara normal dari interaksi user atau data eksternal, gunakan `Result`.

---

## Latihan

1. **Latihan dasar**: Buat fungsi `validate_ticket_title(title: &str) -> Result<(), AppError>` yang return `Err(AppError::ValidationError(...))` kalau `title` kosong atau lebih dari 100 karakter.

2. **Latihan operator `?`**: Buat fungsi `create_ticket(title: &str, user_role: &str) -> Result<String, AppError>` yang memanggil `validate_ticket_title` dengan `?`, lalu cek kalau `user_role == "banned"` return `Unauthorized`, dan kalau lolos semua return `Ok("Ticket berhasil dibuat".to_string())`.

3. **Latihan match**: Di `main()`, panggil `create_ticket` dengan berbagai kombinasi input (judul kosong, user banned, input valid) dan tangani hasilnya dengan `match`.

**Bonus:** Tambahkan varian baru di `AppError` untuk `DatabaseError(String)` dan coba propagasikan lewat `?`.

---

Dengan ini kamu sudah paham cara Rust memaksa kamu berpikir tentang error sejak awal. Itulah kenapa kode Rust cenderung lebih robust: compiler tidak biarkan kamu mengabaikan kemungkinan gagal. Di bab berikutnya kita masuk ke **traits**, fondasi untuk menulis kode yang fleksibel dan reusable.
