# Bab 06: Control Flow

Program komputer perlu tahu: "kalau kondisi ini, lakukan itu. Kalau tidak, lakukan yang lain." Inilah **control flow**: cara mengendalikan alur jalannya program. Di Rust, control flow punya beberapa keunikan dibanding bahasa lain yang menarik untuk diperhatikan.

[ILUSTRASI: diagram jalan bercabang dengan rambu lalu lintas — satu jalur berlabel "if", satu berlabel "else", dan ada putaran balik berlabel "loop"]

---

## if / else if / else

`if` dipakai untuk membuat program mengambil keputusan. Analoginya: petugas helpdesk yang lihat tiket masuk, kalau "server mati", langsung eskalasi. Kalau "lupa password", kasih panduan reset. Kalau yang lain, catat dulu.

```rust
fn main() {
    let issue_type = "lupa password";

    if issue_type == "server mati" {
        println!("Eskalasi ke tim infrastruktur!");
    } else if issue_type == "lupa password" {
        println!("Kirim panduan reset password.");
    } else {
        println!("Catat tiket, tim akan follow up.");
    }
}
```

Aturan penting: kondisi di dalam `if` **harus bertipe `bool`** (true/false). Rust tidak pakai angka 0/1 seperti bahasa lain.

```rust
// ✅ Benar
if is_admin { ... }

// ❌ Error di Rust — angka bukan bool
if 1 { ... }
```

---

## if Sebagai Expression (Unik di Rust!)

Di kebanyakan bahasa, `if` adalah **statement**, cuma menjalankan sesuatu. Di Rust, `if` adalah **expression**, bisa menghasilkan nilai dan langsung disimpan ke variabel. Expression adalah sesuatu yang menghasilkan nilai, sama seperti `3 + 4` yang hasilnya `7`.

```rust
fn main() {
    let ticket_count = 5;

    // if sebagai expression — hasilnya langsung disimpan ke variabel
    let status = if ticket_count > 0 { "Ada ticket" } else { "Tidak ada ticket" };

    println!("{}", status);
}
```

Perhatikan: tidak ada titik koma di dalam blok `{ "Ada ticket" }`. Kalau ada titik koma, nilainya hilang. Baris terakhir tanpa titik koma = nilai yang dikembalikan.

Syaratnya: kedua cabang (`if` dan `else`) **harus return tipe yang sama**. Tidak bisa satu return `&str` dan satunya return angka.

---

## loop: Infinite Loop

`loop` mengulang terus sampai ada yang menghentikan. Ini adalah **infinite loop**, kecuali kita pakai `break` untuk keluar. Analoginya: mesin auto-reply email yang terus jalan sampai ada yang mematikannya.

```rust
fn main() {
    let mut attempt = 0;

    loop {
        attempt += 1;
        println!("Mencoba koneksi ke server... (percobaan ke-{})", attempt);

        if attempt == 3 {
            println!("Berhasil terhubung!");
            break; // keluar dari loop
        }
    }
}
```

Yang keren: `break` di Rust bisa **mengembalikan nilai**.

```rust
fn main() {
    let mut counter = 0;

    let hasil = loop {
        counter += 1;
        if counter == 5 {
            break counter * 2; // nilai ini jadi nilai dari loop
        }
    };

    println!("Hasil: {}", hasil); // Hasil: 10
}
```

---

## while: Loop Bersyarat

`while` artinya "selama kondisi ini benar, terus lakukan". Begitu kondisi salah, loop berhenti. Analoginya: timer penutupan tiket otomatis, "selama waktu belum habis, countdown terus."

```rust
fn main() {
    let mut remaining = 3;

    while remaining > 0 {
        println!("Menutup ticket dalam {} detik...", remaining);
        remaining -= 1;
    }

    println!("Ticket ditutup.");
}
```

Kapan pakai `loop` vs `while`: pakai `loop` kalau kondisi berhenti ada di dalam loop dan tidak jelas dari awal. Pakai `while` kalau kondisi berhentinya sudah jelas sejak sebelum loop dimulai.

---

## for: Loop Terstruktur

`for` dipakai untuk **iterasi**: melewati setiap item dalam kumpulan data satu per satu.

### Range (rentang angka)

```rust
fn main() {
    // 0..5 = dari 0 sampai 4 (eksklusif — angka akhir tidak termasuk)
    for i in 0..5 {
        println!("Nomor: {}", i);
    }

    // 0..=5 = dari 0 sampai 5 (inklusif — angka akhir termasuk)
    for i in 0..=5 {
        println!("Nomor: {}", i);
    }
}
```

### Iterasi Collection

```rust
fn main() {
    let tickets = ["Bug login", "Error 500", "Lupa password"];

    // &tickets atau tickets.iter() — pinjam datanya, tidak mengambil kepemilikan
    for ticket in &tickets {
        println!("Tiket: {}", ticket);
    }
}
```

Tanda `&` di depan `tickets` artinya kita **meminjam** data, bukan mengambilnya. Ini konsep *borrowing* yang dibahas lebih dalam di bab berikutnya.

### Iterasi dengan Index

Kalau butuh nomor urut sekalian, pakai `.iter().enumerate()`:

```rust
fn main() {
    let tickets = ["Bug login", "Error 500", "Lupa password"];

    for (i, ticket) in tickets.iter().enumerate() {
        println!("Ticket {}: {}", i + 1, ticket);
    }
}
```

`.enumerate()` membungkus setiap item dengan nomor urutnya, hasilnya berupa pasangan `(index, item)`.

---

## continue dan break

`break` keluar dari loop sepenuhnya. `continue` skip sisa iterasi ini dan langsung lanjut ke iterasi berikutnya.

```rust
fn main() {
    let tickets = [1, 2, 3, 4, 5];

    for id in tickets {
        if id == 3 {
            continue; // skip ticket nomor 3
        }
        if id == 5 {
            break; // berhenti total di ticket nomor 5
        }
        println!("Memproses ticket #{}", id);
    }
}
// Output: Memproses ticket #1, #2, #4
```

---

## Nested Loop dengan Label

Nested loop adalah loop di dalam loop. Masalahnya: `break` di loop dalam hanya keluar dari loop dalam, bukan loop luar. Rust menyelesaikan ini dengan **label loop**, nama untuk loop yang ditulis dengan tanda kutip tunggal di depan: `'nama_loop`.

```rust
fn main() {
    let categories = ["Billing", "Teknis", "Umum"];
    let priorities = [1, 2, 3];

    'outer: for category in categories {
        for priority in priorities {
            if category == "Teknis" && priority == 2 {
                println!("Ditemukan! {} - Priority {}", category, priority);
                break 'outer; // keluar dari SEMUA loop, bukan cuma yang dalam
            }
            println!("Cek: {} - Priority {}", category, priority);
        }
    }

    println!("Selesai.");
}
```

Tanpa label, `break` hanya keluar dari loop `priorities`. Dengan `break 'outer`, program keluar langsung dari loop `categories` juga.

---

## Contoh: Check Priority Ticket

```rust
fn check_priority(score: u32) -> &str {  // return string literal
    if score >= 9 {
        "Kritis"
    } else if score >= 7 {
        "Tinggi"
    } else if score >= 4 {
        "Sedang"
    } else {
        "Rendah"
    }
}

fn main() {
    let ticket_count = 5;
    let status = if ticket_count > 0 { "Ada ticket" } else { "Tidak ada ticket" };
    println!("Status: {}", status);

    let tickets = ["Bug login", "Error 500", "Lupa password"];
    for (i, ticket) in tickets.iter().enumerate() {
        println!("Ticket {}: {}", i + 1, ticket);
    }

    let mut remaining = 3;
    while remaining > 0 {
        println!("Menutup ticket dalam {} detik...", remaining);
        remaining -= 1;
    }

    let scores = [3, 6, 8, 10];
    for score in scores {
        println!("Score {}: {}", score, check_priority(score));
    }
}
```

Perhatikan fungsi `check_priority`, tidak ada `return` eksplisit dan tidak ada titik koma di baris terakhir tiap cabang. Itulah cara Rust mengembalikan nilai dari function dan expression secara bersamaan.

[ILUSTRASI: flowchart fungsi `check_priority` — kotak score masuk, lalu percabangan >= 9, >= 7, >= 4, dan else, masing-masing mengarah ke label prioritas]

---

## Latihan

1. **Latihan `if` expression**: Buat variabel `role` berisi string `"admin"` atau `"user"`. Pakai `if` expression untuk set variabel `akses` jadi `"penuh"` kalau admin, `"terbatas"` kalau bukan. Print hasilnya.

2. **Latihan `for` + `continue`**: Buat array berisi 5 nama tiket. Loop semua tiket, tapi skip (pakai `continue`) tiket yang panjang namanya kurang dari 8 karakter. Print tiket yang tidak di-skip.

3. **Latihan `loop` dengan return nilai**: Buat `loop` yang menghitung dari 1 sampai ketemu angka yang habis dibagi 7 pertama kali. Simpan angka itu ke variabel dengan `break nilai`, lalu print hasilnya.

4. **Tantangan nested loop**: Buat dua array, satu berisi nama kategori (`["Billing", "Teknis"]`) dan satu berisi level prioritas (`[1, 2, 3]`). Loop keduanya, tapi berhenti total (pakai label) begitu ketemu kategori `"Teknis"` dengan prioritas `3`.

Jalankan dengan `cargo run` dan perhatikan outputnya. Kalau ada error, baca pesannya, Rust biasanya kasih petunjuk yang cukup jelas tentang apa yang salah.
