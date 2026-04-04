# Bab 11: Enum dan Pattern Matching

Bayangkan kamu ngisi formulir tiket support online. Di bagian "Status", kamu nggak bisa nulis sembarangan. Kamu cuma bisa pilih dari daftar yang sudah tersedia: *Open*, *In Progress*, *Resolved*, atau *Closed*. Nggak ada opsi "mungkin" atau "status custom buatan sendiri".

Itulah inti dari **enum** di Rust. Kita mendefinisikan sendiri daftar pilihan yang valid, dan Rust memastikan kode kita hanya menggunakan pilihan dari daftar itu.

---

## Enum Dasar

**Enum** (kependekan dari *enumeration*) adalah tipe data yang nilainya hanya bisa salah satu dari daftar variant yang sudah kita tentukan.

[ILUSTRASI: Dropdown menu "Pilih Role" di form dengan tiga opsi: Admin, Agent, Customer — tidak bisa diisi teks bebas]

Contoh: setiap pengguna di sebuah aplikasi punya role yang hanya tiga kemungkinan: Admin, Agent, atau Customer.

```rust
#[derive(Debug)]
enum Role {
    Admin,
    Agent,
    Customer,
}
```

`#[derive(Debug)]` adalah atribut yang kita tambahkan supaya Rust tahu cara menampilkan nilai enum ini di terminal (misalnya dengan `println!("{:?}", role)`). Tanpa ini, nilai enum tidak bisa di-print.

Cara pakai:

```rust
let role = Role::Agent;
```

`Role::Agent` artinya "nilai `Agent` dari tipe `Role`". Penulisan dua titik dua (`::`) adalah cara Rust mengakses variant dari sebuah enum.

---

## Enum dengan Data

Enum di Rust bukan sekadar label. Setiap variant bisa **membawa data** di dalamnya, dan ini yang membuat enum Rust jauh lebih powerful dibanding bahasa lain.

Contoh: status tiket. Status `InProgress` perlu tahu siapa agent yang menangani. Status `Resolved` perlu menyimpan catatan resolusinya.

```rust
#[derive(Debug)]
enum TicketStatus {
    Open,
    InProgress { agent_id: u32 },
    Resolved { resolution: String },
    Closed,
}
```

`u32` adalah tipe bilangan bulat positif (0 sampai sekitar 4 miliar), cocok untuk ID. `String` adalah teks biasa.

Cara membuat nilai dengan data:

```rust
let status = TicketStatus::InProgress { agent_id: 5 };
let selesai = TicketStatus::Resolved {
    resolution: String::from("Bug sudah dipatch di versi terbaru"),
};
```

Satu tipe `TicketStatus` bisa merepresentasikan semua kemungkinan status beserta datanya, rapi dalam satu tempat.

---

## `match`: Switch yang Lebih Canggih

Setelah punya enum, kita perlu cara untuk memproses setiap variant-nya. Di Rust, kita pakai **`match`**.

Analoginya seperti petugas loket yang melihat nomor antrian kamu, lalu memutuskan ke mana kamu harus pergi. Setiap nomor (variant) punya jalur sendiri.

```rust
fn describe_status(status: &TicketStatus) -> String {
    match status {
        TicketStatus::Open => String::from("Menunggu ditangani"),
        TicketStatus::InProgress { agent_id } => {
            format!("Sedang dikerjakan oleh agent #{}", agent_id)
        }
        TicketStatus::Resolved { resolution } => {
            format!("Selesai: {}", resolution)
        }
        TicketStatus::Closed => String::from("Ditutup"),
    }
}
```

`match` bersifat **exhaustive**: Rust memaksa kita menulis case untuk *semua* variant. Kalau ada yang ketinggalan, kode tidak akan bisa dikompilasi. Ini fitur keamanan, bukan bug. Di variant yang punya data seperti `InProgress { agent_id }`, kita bisa langsung "unpack" datanya di dalam pola match. `&TicketStatus` artinya kita meminjam nilai, bukan mengambil kepemilikannya.

---

## Wildcard dengan `_`

Kadang kita cuma peduli dengan satu atau dua variant, dan sisanya mau diperlakukan sama. Gunakan `_` sebagai wildcard:

```rust
fn is_active(status: &TicketStatus) -> bool {
    match status {
        TicketStatus::Open | TicketStatus::InProgress { .. } => true,
        _ => false,
    }
}
```

`..` di dalam `InProgress { .. }` artinya "abaikan semua field di dalamnya". `_` di baris terakhir menangkap `Resolved` dan `Closed` sekaligus. Tanda `|` di antara dua pola berarti "atau", jadi `Open` atau `InProgress` akan masuk ke branch yang sama.

---

## `if let`: Shorthand match

Kalau kamu hanya perlu menangani *satu* variant dan mengabaikan sisanya, menulis `match` lengkap terasa berlebihan. Gunakan `if let`:

```rust
let status = TicketStatus::InProgress { agent_id: 5 };

if let TicketStatus::InProgress { agent_id } = status {
    println!("Tiket sedang ditangani agent #{}", agent_id);
}
```

`if let` artinya: "kalau nilainya cocok dengan pola ini, jalankan blok kode dan unpack datanya." Kalau nggak cocok, blok diabaikan.

---

## `Option<T>`: Nilai yang Mungkin Tidak Ada

[ILUSTRASI: Kolom "Assigned Agent" di tabel tiket — bisa berisi nama agent, atau kosong (dash) kalau belum ditugaskan]

Di banyak bahasa, nilai yang "mungkin tidak ada" direpresentasikan dengan `null`. Masalahnya, `null` sering jadi sumber bug: kode mencoba mengakses nilai yang ternyata `null` dan program crash.

Rust tidak punya `null`. Sebagai gantinya, ada **`Option<T>`**, yaitu sebuah enum bawaan Rust:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

`T` adalah placeholder tipe yang bisa diganti tipe apa saja. `Some(T)` artinya "ada nilainya", `None` artinya "tidak ada nilai".

Contoh:

```rust
let assigned_agent: Option<u32> = Some(5);
let unassigned: Option<u32> = None;
```

Untuk menggunakan nilainya, kita *harus* menangani kedua kemungkinan:

```rust
if let Some(agent_id) = assigned_agent {
    println!("Ditangani agent #{}", agent_id);
}

match unassigned {
    Some(id) => println!("Agent: {}", id),
    None => println!("Belum ada agent yang ditugaskan"),
}
```

Ini lebih aman dari `null` karena Rust *tidak mengizinkan* kamu mengakses nilai di dalam `Option` langsung tanpa memeriksa dulu apakah isinya `Some` atau `None`. Compiler akan protes sebelum kode sempat jalan.

---

## Method pada Enum

Seperti struct, enum juga bisa punya method menggunakan `impl`. Ini cara kita menaruh logika yang berkaitan langsung di dalam tipe-nya.

```rust
impl Role {
    fn can_view_all_tickets(&self) -> bool {
        match self {
            Role::Admin | Role::Agent => true,
            Role::Customer => false,
        }
    }

    fn label(&self) -> &str {
        match self {
            Role::Admin => "Administrator",
            Role::Agent => "Support Agent",
            Role::Customer => "Customer",
        }
    }
}
```

`&self` artinya method ini meminjam nilai enum, tidak mengambil kepemilikannya. `&str` adalah tipe string yang "dipinjam" (bukan `String` yang dimiliki penuh).

Cara pakai:

```rust
let role = Role::Agent;
println!("Role: {}", role.label());
println!("Bisa lihat semua ticket: {}", role.can_view_all_tickets());
```

Output:
```
Role: Support Agent
Bisa lihat semua ticket: true
```

---

## Kode Lengkap

```rust
#[derive(Debug)]
enum Role {
    Admin,
    Agent,
    Customer,
}

#[derive(Debug)]
enum TicketStatus {
    Open,
    InProgress { agent_id: u32 },
    Resolved { resolution: String },
    Closed,
}

impl Role {
    fn can_view_all_tickets(&self) -> bool {
        match self {
            Role::Admin | Role::Agent => true,
            Role::Customer => false,
        }
    }

    fn label(&self) -> &str {
        match self {
            Role::Admin => "Administrator",
            Role::Agent => "Support Agent",
            Role::Customer => "Customer",
        }
    }
}

fn describe_status(status: &TicketStatus) -> String {
    match status {
        TicketStatus::Open => String::from("Menunggu ditangani"),
        TicketStatus::InProgress { agent_id } => {
            format!("Sedang dikerjakan oleh agent #{}", agent_id)
        }
        TicketStatus::Resolved { resolution } => {
            format!("Selesai: {}", resolution)
        }
        TicketStatus::Closed => String::from("Ditutup"),
    }
}

fn main() {
    let role = Role::Agent;
    println!("Role: {}", role.label());
    println!("Bisa lihat semua ticket: {}", role.can_view_all_tickets());

    let status = TicketStatus::InProgress { agent_id: 5 };
    println!("{}", describe_status(&status));

    let assigned_agent: Option<u32> = Some(5);
    let unassigned: Option<u32> = None;

    if let Some(agent_id) = assigned_agent {
        println!("Ditangani agent #{}", agent_id);
    }

    match unassigned {
        Some(id) => println!("Agent: {}", id),
        None => println!("Belum ada agent yang ditugaskan"),
    }
}
```

---

## Latihan

1. **Tambah variant baru.** Tambahkan variant `OnHold { reason: String }` ke `TicketStatus`. Pastikan semua `match` yang ada masih bisa dikompilasi: kamu harus tambahkan case baru di setiap `match` yang menggunakan `TicketStatus`.

2. **Buat method baru.** Di `impl Role`, buat method `can_close_ticket(&self) -> bool` yang mengembalikan `true` hanya untuk `Admin` dan `Agent`.

3. **Gunakan `Option`.** Buat fungsi `get_agent_id(status: &TicketStatus) -> Option<u32>` yang mengembalikan `Some(agent_id)` kalau statusnya `InProgress`, dan `None` untuk status lainnya.

4. **Eksperimen `if let`.** Panggil fungsi dari latihan 3, lalu gunakan `if let` untuk mencetak ID agent-nya kalau ada.

> Kunci keberhasilan: jalankan `cargo build` setelah setiap perubahan. Biarkan compiler Rust jadi teman belajarmu, pesan error-nya sangat membantu!
