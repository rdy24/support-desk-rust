# Bab 13: Traits

Bayangkan kamu rekrut staf baru untuk tim. Kamu nggak peduli orangnya lulusan mana, pengalaman berapa tahun, atau dari kota mana. Yang penting satu: **dia bisa bikin laporan harian**. Selama bisa bikin laporan, dia lolos seleksi.

Itulah trait di Rust. Bukan soal "siapa kamu" (class / warisan), tapi soal **"apa yang bisa kamu lakukan"**. Trait adalah kontrak kemampuan.

---

## Apa Itu Trait?

Trait (dibaca: "treyt") adalah cara Rust mendefinisikan **sekumpulan kemampuan** yang bisa dimiliki oleh berbagai tipe data.

Kalau kamu pernah dengar kata *interface* di bahasa lain (Java, TypeScript, Go), trait itu mirip. Bedanya, trait di Rust lebih powerful karena bisa punya implementasi default.

Yang penting dipahami: **trait bukan pewarisan** (*inheritance*). Di Rust, kita tidak bilang "Ticket adalah turunan dari Item". Kita bilang "Ticket bisa melakukan Summarize". Perbedaan ini kecil tapi krusial. Rust tidak punya class hierarchy.

[ILUSTRASI: Dua kolom perbandingan. Kiri: "Inheritance (OOP lama)" dengan panah dari parent ke child. Kanan: "Trait (Rust)" dengan beberapa struct berbeda (Ticket, User, Report) semua menunjuk ke satu kontrak Summarize.]

---

## Mendefinisikan Trait

Sintaksnya pakai kata kunci `trait`:

```rust
trait Summarize {
    fn summary(&self) -> String;
}
```

Kita bilang: "siapapun yang claim bisa `Summarize`, dia HARUS punya method `summary` yang return `String`." Tapi belum ada kata *gimana* caranya, itu urusan masing-masing tipe.

Satu trait bisa punya banyak method. Method-method itu boleh hanya deklarasi (tanpa isi) atau punya implementasi default.

---

## Implementasi Trait

Setelah trait didefinisikan, kita `impl` trait itu untuk tipe tertentu:

```rust
use std::fmt;

trait Summarize {
    fn summary(&self) -> String;

    fn short_summary(&self) -> String {
        let s = self.summary();
        let end = 50.min(s.len());
        format!("{}...", &s[..end])
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Ticket {
    id: u32,
    title: String,
    status: String,
}

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    role: String,
}

impl Summarize for Ticket {
    fn summary(&self) -> String {
        format!("[#{}] {} ({})", self.id, self.title, self.status)
    }
}

impl Summarize for User {
    fn summary(&self) -> String {
        format!("User #{}: {} ({})", self.id, self.name, self.role)
    }
}
```

Kata kunci kuncinya: `impl NamaTrait for NamaStruct`. Setiap struct mengisi kontrak dengan caranya sendiri. `Ticket` meringkas tiket, `User` meringkas data user. Logika berbeda, kontrak sama.

---

## Default Method

Perhatikan `short_summary` di atas. Method itu punya **isi langsung** di dalam trait:

```rust
fn short_summary(&self) -> String {
    let s = self.summary();
    let end = 50.min(s.len());
    format!("{}...", &s[..end])
}
```

Ini disebut **default method**. Kalau struct yang implement trait ini tidak mendefinisikan `short_summary` sendiri, dia otomatis pakai versi default ini. Ibaratnya: kontrak kerja menyediakan template laporan. Boleh dipakai apa adanya, boleh juga dibuat versi sendiri.

Perhatikan: kita menyimpan hasil `self.summary()` ke variabel `s` terlebih dahulu, lalu menggunakannya dua kali. Ini lebih efisien daripada memanggil `self.summary()` dua kali, karena setiap pemanggilan method membuat string baru.

Struct tetap **boleh override** default method kalau mau perilaku berbeda.

---

## Trait Bound

Dengan trait bound, kita bisa buat fungsi yang menerima **tipe apapun, selama implement trait tertentu**:

```rust
fn print_summary(item: &impl Summarize) {
    println!("{}", item.summary());
}
```

`impl Summarize` di sini artinya: "terima parameter apapun yang implement `Summarize`". Passing `&Ticket` atau `&User` ke fungsi ini keduanya valid.

Ada dua cara penulisan yang setara:

```rust
// Cara 1: syntax `impl Trait` — lebih ringkas
fn print_summary(item: &impl Summarize) {
    println!("{}", item.summary());
}

// Cara 2: syntax generic dengan trait bound — lebih eksplisit
fn print_summary<T: Summarize>(item: &T) {
    println!("{}", item.summary());
}
```

Keduanya menghasilkan perilaku yang sama. Cara 2 lebih berguna kalau kamu butuh nama tipe `T` di beberapa tempat dalam satu fungsi. Untuk pemula, cara 1 lebih mudah dibaca.

[ILUSTRASI: Fungsi `print_summary` sebagai "loket kasir" yang menerima siapapun yang punya "kartu member" (trait Summarize). Ticket datang → diterima. User datang → diterima. Tipe lain tanpa kartu → ditolak compiler.]

---

## derive: Trait Gratis

Beberapa trait sangat umum sehingga Rust menyediakan cara otomatis untuk generate implementasinya. Cukup tulis `#[derive(...)]` di atas struct:

```rust
#[derive(Debug, Clone, PartialEq)]
struct Ticket {
    id: u32,
    title: String,
    status: String,
}
```

`#[derive(...)]` adalah **attribute**: instruksi ke compiler untuk generate kode tertentu secara otomatis. Tanpa ini, kita harus tulis `impl Debug for Ticket { ... }` sendiri secara manual.

| Trait | Fungsi |
|-------|--------|
| `Debug` | Bisa dicetak dengan `{:?}` untuk debugging |
| `Clone` | Bisa di-copy dengan `.clone()` |
| `PartialEq` | Bisa dibandingkan dengan `==` dan `!=` |

Syarat `derive` bisa dipakai: **semua field di dalam struct juga harus implement trait yang sama**. `String`, `u32`, dan tipe primitif lainnya sudah implement `Debug`, `Clone`, `PartialEq`. Jadi aman.

---

## Trait dari Standard Library

Rust punya banyak trait bawaan yang sangat sering dipakai. Beberapa yang paling penting:

### `Debug`

Untuk mencetak struct ke output dengan format debug. Diaktifkan lewat `#[derive(Debug)]`.

```rust
let ticket = Ticket { id: 1, title: String::from("Login error"), status: String::from("open") };
println!("{:?}", ticket);   // format debug
println!("{:#?}", ticket);  // format debug yang lebih rapi (pretty print)
```

### `Display`

Untuk mencetak tipe dengan format "ramah pengguna". Tidak bisa di-derive, harus ditulis manual karena Rust tidak tahu format yang kamu inginkan:

```rust
impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[#{}] {}", self.id, self.title)
    }
}
```

Setelah ini, `println!("{}", ticket)` bisa dipakai.

`fmt::Display` dan `fmt::Formatter` adalah bagian dari modul `std::fmt` (standard library Rust untuk formatting). Perlu `use std::fmt;` di bagian atas file.

### `Clone`

Membuat salinan (deep copy) dari sebuah nilai:

```rust
let ticket2 = ticket.clone();
```

Tanpa `Clone`, kita tidak bisa memanggil `.clone()`.

### `PartialEq`

Mengaktifkan operator `==` dan `!=`:

```rust
println!("Sama? {}", ticket == ticket2); // true
```

"Partial" dalam `PartialEq` artinya: ada nilai yang tidak bisa dibandingkan secara bermakna (seperti `NaN` di floating point). Untuk kebutuhan sehari-hari, `PartialEq` sudah lebih dari cukup.

---

## Trait sebagai Return Type

Trait juga bisa dipakai sebagai tipe kembalian fungsi:

```rust
fn buat_item(is_ticket: bool) -> impl Summarize {
    if is_ticket {
        Ticket {
            id: 99,
            title: String::from("Bug kritis"),
            status: String::from("open"),
        }
    } else {
        // ⚠️ Kalau di sini kita return tipe lain (misal User),
        // kode ini TIDAK akan compile karena `impl Trait`
        // hanya boleh return satu tipe konkret.
        // Solusinya: Box<dyn Summarize> — dibahas nanti.
        panic!("Hanya untuk contoh")
    }
}
```

`impl Summarize` sebagai return type artinya: "fungsi ini akan return sesuatu yang implement `Summarize`, tapi kamu tidak perlu tahu tipe persisnya."

Batasan penting: `impl Trait` sebagai return type hanya bisa untuk **satu tipe konkret**. Kalau kamu perlu return `Ticket` di satu branch dan `User` di branch lain, butuh `Box<dyn Trait>`. Itu topik Bab selanjutnya tentang trait objects.

---

## Kode Lengkap

Gabungan semua konsep di atas:

```rust
use std::fmt;

trait Summarize {
    fn summary(&self) -> String;

    fn short_summary(&self) -> String {
        let s = self.summary();
        let end = 50.min(s.len());
        format!("{}...", &s[..end])
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Ticket {
    id: u32,
    title: String,
    status: String,
}

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    role: String,
}

impl Summarize for Ticket {
    fn summary(&self) -> String {
        format!("[#{}] {} ({})", self.id, self.title, self.status)
    }
}

impl Summarize for User {
    fn summary(&self) -> String {
        format!("User #{}: {} ({})", self.id, self.name, self.role)
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[#{}] {}", self.id, self.title)
    }
}

fn print_summary(item: &impl Summarize) {
    println!("{}", item.summary());
}

fn main() {
    let ticket = Ticket {
        id: 1,
        title: String::from("Login tidak bisa"),
        status: String::from("open"),
    };

    let user = User {
        id: 42,
        name: String::from("Budi"),
        role: String::from("customer"),
    };

    print_summary(&ticket);
    print_summary(&user);
    println!("Display: {}", ticket);

    let ticket2 = ticket.clone();
    println!("Sama? {}", ticket == ticket2);
}
```

Output yang diharapkan:
```
[#1] Login tidak bisa (open)
User #42: Budi (customer)
Display: [#1] Login tidak bisa
Sama? true
```

---

## Latihan

1. **Tambah trait `Prioritize`** dengan method `priority_level(&self) -> u8` yang return angka 1–5. Implementasikan untuk `Ticket`. Misalnya jika status `"open"` return 3, jika `"urgent"` return 5.

2. **Buat struct `Report`** dengan field `title: String` dan `content: String`. Implementasikan trait `Summarize` untuk `Report`.

3. **Buat fungsi `log_all`** yang menerima `Vec` berisi item-item yang implement `Summarize`, lalu cetak `short_summary()` dari setiap item.
   Hint: signature-nya `fn log_all(items: &[&impl Summarize])`. Atau coba `Vec<Box<dyn Summarize>>` kalau kamu penasaran (kita bahas ini di Bab 14).

4. **Tambahkan `#[derive(PartialEq)]` ke `User`** dan coba bandingkan dua user. Apa yang terjadi?

---

Trait adalah fondasi dari banyak hal di Rust. Mulai dari error handling, iterator, hingga async. Kalau konsep "kontrak kemampuan" ini sudah masuk, bab-bab selanjutnya akan terasa jauh lebih masuk akal.
