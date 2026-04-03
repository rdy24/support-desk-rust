# Bab 33: Testing - Unit Test

Kamu sudah nulis kode, sudah jalan, tapi yakin nggak kalau kode kamu benar? Bab ini bahas cara bikin test otomatis di Rust, bawaan, tanpa library tambahan.

---

## Kenapa Test Itu Penting?

Bayangkan pabrik sepatu. Sebelum sepatu dikirim ke toko, ada tim quality control (QC) yang ngecek satu per satu: solnya kuat nggak? Jahitannya rapi nggak? Ukurannya sesuai nggak?

Kalau nggak ada QC, sepatu yang cacat bisa lolos dan nyampe ke pelanggan. Komplen datang, reputasi rusak, kerugian numpuk.

Unit test itu peran QC-nya kode kamu. Setiap kali kamu ubah sesuatu, test otomatis ngecek apakah "produk" kamu masih sesuai standar. Kalau ada yang rusak, test langsung teriak sebelum kode naik ke production.

[ILUSTRASI: Diagram alur pabrik sepatu — produksi → QC → pengiriman, di bawahnya ada analogi: nulis kode → jalankan test → deploy]

Tanpa test, kamu ubah satu fungsi dan nggak tau apakah fungsi lain ikut rusak. Bug ketemu di production, mahal diperbaiki dan memalukan. Dengan test, kamu ubah fungsi, jalankan `cargo test`, dan langsung tau ada yang rusak atau nggak. Bug ketemu di local, murah, cepat, aman.

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 31:**
- UserService dan DashboardService lengkap dengan business logic sudah di "Hasil Akhir Bab 31"
- Semuanya ready untuk di-integrate di handlers

**State Sebelumnya:**
Dari Bab 31, semua service layer lengkap. Aplikasi sudah bisa dijalankan end-to-end. Bab 33-34 fokus ke testing untuk memastikan semuanya bekerja dengan benar.

---

## Unit Test Bawaan Rust

Kabar bagus: Rust punya sistem testing bawaan. Kamu **tidak perlu install library tambahan** untuk bikin unit test dasar. Cukup tulis kode, tambahkan modul test, langsung jalan.

Ini beda dari banyak bahasa lain yang butuh framework seperti JUnit (Java), pytest (Python), atau Jest (JavaScript). Di Rust, semuanya sudah ada di toolchain standar.

---

## Anatomy Test: `#[cfg(test)]`, `#[test]`, dan `assert!`

Tiga komponen utama unit test di Rust.

### `#[cfg(test)]`

Ini adalah **attribute**, semacam instruksi untuk compiler. `cfg(test)` artinya: "blok kode ini hanya dikompilasi saat mode test". Jadi kode test kamu tidak ikut masuk ke binary production.

```rust
#[cfg(test)]
mod tests {
    // semua test ada di sini
}
```

`mod tests` adalah nama konvensi, boleh nama lain, tapi `tests` sudah jadi standar komunitas.

### `#[test]`

Setiap fungsi yang mau dijadikan test harus dikasih attribute `#[test]` di atasnya. Ini cara kamu bilang ke Rust: "fungsi ini adalah test, bukan fungsi biasa".

```rust
#[test]
fn test_penjumlahan() {
    let hasil = 2 + 2;
    assert_eq!(hasil, 4);
}
```

### `assert!`, `assert_eq!`, `assert_ne!`

Ini adalah macro untuk mengecek kondisi. Kalau kondisi gagal, test langsung gagal dengan pesan error.

| Macro | Arti |
|---|---|
| `assert!(kondisi)` | Pastikan kondisi bernilai `true` |
| `assert_eq!(a, b)` | Pastikan `a` sama dengan `b` |
| `assert_ne!(a, b)` | Pastikan `a` **tidak** sama dengan `b` |

Contoh:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_nama_lengkap() {
        let nama = format!("{} {}", "Budi", "Santoso");
        assert_eq!(nama, "Budi Santoso");
    }

    #[test]
    fn test_password_tidak_sama() {
        let password = "rahasia123";
        let typo = "rahasia124";
        assert_ne!(password, typo);
    }

    #[test]
    fn test_list_tidak_kosong() {
        let items = vec![1, 2, 3];
        assert!(!items.is_empty());
    }
}
```

### `#[should_panic]`

Kadang kamu mau test bahwa suatu fungsi **memang harus panic** dalam kondisi tertentu. Gunakan `#[should_panic]`.

```rust
fn bagi(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Tidak bisa bagi dengan nol!");
    }
    a / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Tidak bisa bagi dengan nol")]
    fn test_bagi_dengan_nol() {
        bagi(10, 0); // harus panic
    }
}
```

`expected` di `should_panic` opsional, tapi dianjurkan agar kamu tau panic-nya memang yang diharapkan, bukan panic lain yang nggak sengaja.

---

## Test yang Return `Result`

Selain `assert!`, kamu bisa bikin test yang return `Result`. Kalau fungsi return `Err`, test dianggap gagal. Ini berguna kalau kamu mau pakai `?` operator di dalam test.

```rust
use std::error::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_angka() -> Result<(), Box<dyn Error>> {
        let angka: i32 = "42".parse()?;
        assert_eq!(angka, 42);
        Ok(())
    }
}
```

Pola `-> Result<(), Box<dyn Error>>` memungkinkan kamu pakai `?` di dalam test, jauh lebih bersih dibanding `.unwrap()` di mana-mana.

---

## Menjalankan Test

### Semua test

```bash
cargo test
```

Output-nya kira-kira:

```
running 4 tests
test tests::test_nama_lengkap ... ok
test tests::test_password_tidak_sama ... ok
test tests::test_list_tidak_kosong ... ok
test tests::test_parse_angka ... ok

test result: ok. 4 passed; 0 failed
```

### Satu test spesifik

```bash
cargo test test_nama_lengkap
```

Berguna kalau kamu lagi debug satu test dan nggak mau jalankan semuanya.

### Test dengan output println

Secara default, output `println!` di dalam test disembunyikan. Untuk lihat:

```bash
cargo test -- --nocapture
```

[ILUSTRASI: Screenshot terminal menunjukkan output `cargo test` dengan semua test hijau (ok), lalu contoh satu test merah (FAILED) dengan pesan assertion error]

---

## Contoh: Unit Test untuk Business Logic

Ini contoh nyata dari project support desk kita. Kita test logika akses tiket (siapa yang boleh lihat tiket siapa) **tanpa perlu database**.

Unit test yang baik itu **isolated**: tidak bergantung ke DB, network, atau state luar. Yang ditest murni logikanya saja.

```rust
// src/services/ticket_service.rs

#[cfg(test)]
mod tests {
    use super::*;

    fn make_claims(role: &str, id: &str) -> Claims {
        Claims {
            sub: id.to_string(),
            email: "test@example.com".to_string(),
            role: role.to_string(),
            exp: 9999999999,
        }
    }

    #[test]
    fn test_check_access_admin_always_allowed() {
        let service = TicketService::new_for_test();
        let ticket = Ticket {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(),
            // ...
        };
        let claims = make_claims("admin", "any-id");
        assert!(service.check_access(&ticket, &claims).is_ok());
    }

    #[test]
    fn test_check_access_customer_own_ticket() {
        let customer_id = Uuid::new_v4();
        let ticket = Ticket {
            id: Uuid::new_v4(),
            customer_id,
            // ...
        };
        let claims = make_claims("customer", &customer_id.to_string());
        let service = TicketService::new_for_test();
        assert!(service.check_access(&ticket, &claims).is_ok());
    }

    #[test]
    fn test_check_access_customer_other_ticket() {
        let ticket = Ticket {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(), // ID lain
            // ...
        };
        let claims = make_claims("customer", &Uuid::new_v4().to_string());
        let service = TicketService::new_for_test();
        assert!(service.check_access(&ticket, &claims).is_err());
    }

    #[test]
    fn test_validate_priority() {
        assert!(validate_priority("urgent").is_ok());
        assert!(validate_priority("low").is_ok());
        assert!(validate_priority("invalid").is_err());
    }
}
```

Ada beberapa pola penting di sini. **`use super::*`** mengimport semua dari module parent (file yang sama), ini cara standar akses kode yang mau ditest. **`make_claims` helper** adalah fungsi bantu untuk bikin data test, bukan test itu sendiri (tidak ada `#[test]`), tapi dipakai oleh test lain untuk hindari duplikasi. **`new_for_test()`** adalah method khusus yang bikin instance service tanpa dependency database, pola umum untuk buat constructor alternatif saat testing. Setiap test punya satu skenario fokus: `test_check_access_admin`, `test_check_access_customer_own`, dan seterusnya.

---

## Tips Menulis Test yang Baik

**1. Isolated:** tidak bergantung ke luar.
Test harus bisa jalan kapan saja, di mana saja, tanpa setup khusus. Jangan bergantung ke database live, API eksternal, atau file sistem kecuali memang perlu.

**2. Deterministic:** hasil selalu sama.
Jalankan 100 kali, hasilnya harus sama. Hindari random, waktu sekarang, atau state global yang bisa berubah.

**3. Readable:** nama test itu dokumentasi.
Nama seperti `test_check_access_customer_other_ticket` langsung cerita apa yang ditest. Hindari nama generik seperti `test1` atau `test_function`.

**4. Satu test, satu assertion utama.**
Boleh ada beberapa `assert!`, tapi fokus pada satu skenario. Kalau test gagal, kamu langsung tau skenario mana yang bermasalah.

**5. Test edge case, bukan cuma happy path.**
`validate_priority("invalid").is_err()` itu penting. Test bahwa input buruk ditolak dengan benar, bukan cuma input bagus diterima.

---

## Latihan

1. Buat fungsi `is_valid_email(email: &str) -> bool` yang return `true` kalau string mengandung `@` dan `.`. Tulis minimal 3 unit test: email valid, email tanpa `@`, email tanpa `.`.

2. Buat fungsi `calculate_discount(price: f64, role: &str) -> f64` dengan aturan: `"vip"` dapat diskon 20%, `"member"` dapat 10%, selain itu tidak dapat diskon. Tulis test untuk setiap role, dan satu test untuk role yang tidak dikenal.

3. Buat fungsi yang panic kalau menerima string kosong. Tulis test dengan `#[should_panic]` untuk memverifikasi panic-nya.

4. Ubah salah satu test kamu agar return `Result<(), Box<dyn Error>>` dan gunakan `?` operator di dalamnya.
