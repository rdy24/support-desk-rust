# Bab 10: Struct dan Method

Sejauh ini kita sudah bisa bekerja dengan variabel, tipe data, fungsi, dan kondisi. Masalah muncul ketika ada banyak data yang saling berkaitan. Data sebuah tiket support misalnya: ada ID-nya, judulnya, statusnya, prioritasnya. Kalau disimpan dalam variabel terpisah, cepat berantakan.

**Struct** adalah cara Rust untuk mengelompokkan data terkait jadi satu paket rapi. Dengan **method**, struct itu bisa punya perilaku sendiri.

---

## Struct = Formulir Isian

Bayangkan formulir laporan kerusakan di kantor. Formulirnya punya kolom: nomor tiket, judul masalah, nama pelapor, tingkat urgensi. Semua kolom itu berkaitan satu sama lain karena mereka semua ngomongin satu tiket yang sama.

**Struct** di Rust persis seperti formulir itu. Kamu mendefinisikan kolom-kolomnya (disebut **field**), lalu setiap kali butuh data tiket, kamu "isi formulirnya" jadi satu paket.

[ILUSTRASI: Formulir tiket support fisik dengan kolom ID, Judul, Status, Prioritas — lalu di sebelahnya kode struct Rust yang memetakan tiap kolom ke field]

---

## Mendefinisikan Struct

Mendefinisikan struct berarti membuat "cetakan formulir" dulu, belum isi datanya, cuma tentukan ada kolom apa saja dan tipenya apa.

```rust
#[derive(Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
    role: String,
}

#[derive(Debug)]
struct Ticket {
    id: u32,
    title: String,
    status: String,
    priority: u8,
    created_by: u32, // id dari user yang membuat tiket
}
```

Kata kunci `struct` diikuti nama struct (huruf besar di awal, konvensi Rust), lalu kurung kurawal berisi daftar field. Setiap field punya nama dan tipe data, dipisah titik dua.

`#[derive(Debug)]` di baris atas struct itu nanti dibahas di bagian tersendiri.

---

## Membuat Instance

Kalau struct adalah cetakan formulir, maka **instance** adalah formulir yang sudah diisi. Instance dibuat dengan menyebut nama struct lalu mengisi tiap field-nya:

```rust
fn main() {
    let user = User {
        id: 42,
        name: String::from("Budi Santoso"),
        email: String::from("budi@example.com"),
        role: String::from("user"),
    };
}
```

Semua field harus diisi karena Rust tidak punya nilai default otomatis. Kalau ada field yang dilewati, Rust langsung kasih error saat kompilasi. Ini bagus: tidak ada data yang "terlupa".

---

## Mengakses Field

Untuk mengakses field dari sebuah instance, pakai **dot notation**:

```rust
println!("Nama: {}", user.name);
println!("Email: {}", user.email);
```

Kalau instance dideklarasikan `mut` (mutable), field-nya juga bisa diubah lewat dot notation:

```rust
let mut user = User {
    id: 42,
    name: String::from("Budi Santoso"),
    email: String::from("budi@example.com"),
    role: String::from("user"),
};

user.role = String::from("admin");
```

---

## Struct Update Syntax

Sering kali kita perlu instance baru yang hampir sama dengan yang sudah ada, cuma beda satu-dua field. Daripada tulis ulang semuanya, Rust punya **struct update syntax** dengan `..`:

```rust
let user_lama = User {
    id: 42,
    name: String::from("Budi Santoso"),
    email: String::from("budi@example.com"),
    role: String::from("user"),
};

let user_baru = User {
    id: 43,
    email: String::from("budi.baru@example.com"),
    ..user_lama  // sisa field diambil dari user_lama
};
```

`..user_lama` artinya "ambil semua field yang belum disebutkan dari `user_lama`". Harus diletakkan paling akhir di dalam kurung kurawal.

> **Catatan**: Field yang dipindahkan pakai `..` mengikuti aturan ownership yang sudah kita pelajari di bab sebelumnya. Field bertipe `String` akan di-move, sehingga `user_lama` tidak bisa dipakai lagi setelah itu (kecuali field `String`-nya kamu override semua).

---

## `impl` Block: Menambah Perilaku

Struct hanya menyimpan data. Tapi di dunia nyata, tiket support tidak cuma "punya data". Ia juga punya **perilaku**: bisa dicek apakah urgent, bisa ditutup, bisa ditampilkan ringkasannya.

Di Rust, perilaku ditambahkan ke struct lewat blok **`impl`** (kependekan dari *implementation*):

```rust
impl Ticket {
    // di sini kita tulis semua fungsi yang "milik" Ticket
}
```

Semua fungsi di dalam `impl Ticket` berasosiasi dengan tipe `Ticket`. Ada dua jenis: **method** dan **associated function**.

---

## Method: `&self` dan `&mut self`

**Method** adalah fungsi yang dipanggil pada sebuah instance, artinya ia perlu tahu "tiket mana yang sedang diproses". Parameter pertamanya selalu `self` dalam salah satu bentuk:

- `&self`: pinjam instance secara immutable (hanya baca)
- `&mut self`: pinjam instance secara mutable (bisa ubah data)

```rust
impl Ticket {
    fn is_urgent(&self) -> bool {
        self.priority >= 8
    }

    fn close(&mut self) {
        self.status = String::from("closed");
    }

    fn summary(&self) -> String {
        format!("[#{}] {} - {} (prioritas: {})", self.id, self.title, self.status, self.priority)
    }
}
```

`is_urgent` dan `summary` hanya membaca data, jadi pakai `&self`. `close` mengubah status tiket, jadi butuh `&mut self` dan instannya harus dideklarasikan sebagai `mut`.

Cara memanggilnya pakai dot notation juga:

```rust
let mut ticket = /* ... */;
println!("{}", ticket.summary());
ticket.close();
```

[ILUSTRASI: Diagram tiket dengan panah ke tiga method — is_urgent (mata/baca), close (kunci/ubah), summary (kertas/baca) — dengan label &self dan &mut self]

---

## Associated Function: Constructor

**Associated function** adalah fungsi di dalam `impl` block tapi *tanpa* parameter `self`. Ia tidak dipanggil pada instance, tapi pada tipe struct-nya langsung dengan sintaks `NamaStruct::nama_fungsi()`.

Penggunaan paling umum adalah sebagai **constructor** untuk membuat instance baru dengan cara yang lebih praktis:

```rust
impl Ticket {
    fn new(id: u32, title: &str, priority: u8, created_by: u32) -> Ticket {
        Ticket {
            id,
            title: title.to_string(),
            status: String::from("open"),
            priority,
            created_by,
        }
    }
}
```

Tidak ada `self` di parameter karena ini associated function, bukan method. `id,` tanpa nilai adalah shorthand `id: id`: kalau nama variabel sama dengan nama field, boleh tulis sekali saja. `title.to_string()` mengkonversi `&str` (string slice) menjadi `String` yang owned.

Cara memanggilnya:

```rust
let ticket = Ticket::new(1, "Login tidak bisa", 9, 42);
```

Jauh lebih ringkas daripada mengisi semua field secara manual, dan status selalu mulai dari `"open"` tanpa perlu diingat tiap kali.

---

## Debug Print dengan `#[derive(Debug)]`

Coba `println!("{}", ticket)`, Rust akan error. Tipe `Ticket` tidak tahu cara menampilkan dirinya sebagai teks biasa.

Solusinya ada dua: implementasi manual (nanti di bab Traits), atau pakai **derive macro** yang Rust sediakan. `#[derive(Debug)]` di atas definisi struct otomatis mengajarkan Rust cara menampilkan struct itu untuk keperluan debugging:

```rust
#[derive(Debug)]
struct Ticket { /* ... */ }
```

Lalu pakai format specifier `{:?}` (atau `{:#?}` untuk tampilan yang lebih rapi dan berlipat):

```rust
println!("{:?}", ticket);
// Output: Ticket { id: 1, title: "Login tidak bisa", status: "open", priority: 9, created_by: 42 }

println!("{:#?}", ticket);
// Output:
// Ticket {
//     id: 1,
//     title: "Login tidak bisa",
//     ...
// }
```

Ini sangat berguna saat debugging karena kamu bisa cek isi struct tanpa perlu tulis fungsi tampilan manual.

---

## Kode Lengkap

```rust
#[derive(Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
    role: String,
}

#[derive(Debug)]
struct Ticket {
    id: u32,
    title: String,
    status: String,
    priority: u8,
    created_by: u32,
}

impl Ticket {
    fn new(id: u32, title: &str, priority: u8, created_by: u32) -> Ticket {
        Ticket {
            id,
            title: title.to_string(),
            status: String::from("open"),
            priority,
            created_by,
        }
    }

    fn is_urgent(&self) -> bool {
        self.priority >= 8
    }

    fn close(&mut self) {
        self.status = String::from("closed");
    }

    fn summary(&self) -> String {
        format!("[#{}] {} - {} (prioritas: {})", self.id, self.title, self.status, self.priority)
    }
}

fn main() {
    let mut ticket = Ticket::new(1, "Login tidak bisa", 9, 42);
    println!("{}", ticket.summary());
    println!("Urgent? {}", ticket.is_urgent());
    ticket.close();
    println!("Setelah ditutup: {}", ticket.summary());
    println!("{:?}", ticket);
}
```

Output yang dihasilkan:

```
[#1] Login tidak bisa - open (prioritas: 9)
Urgent? true
Setelah ditutup: [#1] Login tidak bisa - closed (prioritas: 9)
Ticket { id: 1, title: "Login tidak bisa", status: "closed", priority: 9, created_by: 42 }
```

---

## Latihan

1. **Tambah field `assignee_id: Option<u32>`** ke struct `Ticket` untuk menyimpan ID user yang menangani tiket (boleh kosong/`None` kalau belum di-assign). Update `Ticket::new` agar `assignee_id` mulai dengan `None`.

2. **Buat method `assign(&mut self, user_id: u32)`** yang mengisi `assignee_id` dengan user ID yang diberikan.

3. **Buat method `is_assigned(&self) -> bool`** yang mengembalikan `true` kalau tiket sudah punya assignee.

4. **Buat struct `User`** dengan field `id`, `name`, `email`, dan `role`. Buat associated function `User::new(...)` sebagai constructor-nya.

5. **Bonus**: Buat beberapa tiket, lalu filter mana saja yang urgent (`priority >= 8`) menggunakan loop dan kondisi `if`.

Di bab berikutnya, kita kenalan dengan **Enum**: cara Rust merepresentasikan nilai yang punya beberapa kemungkinan bentuk, seperti status tiket yang bisa `open`, `in_progress`, atau `closed`.
