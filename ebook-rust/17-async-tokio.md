# Bab 17: Async Rust dan Tokio

Bayangkan masuk ke restoran. Pelayan datang, kamu pesan nasi goreng, dan pelayan itu berdiri di meja, diam, nunggu sampai nasi goreng selesai dimasak. 10 menit. Tamu lain di meja sebelah melambaikan tangan minta tolong, tapi pelayan tadi nggak bisa kemana-mana. Itulah yang terjadi di kode **sinkronus**: program berhenti dan nunggu satu operasi selesai sebelum lanjut ke yang lain.

Pelayan yang smart mencatat pesanan, kasih ke dapur, lalu langsung ke meja lain. Dapur selesai masak? Baru pelayan balik bawa makanannya. Sementara itu dia udah melayani 5 meja sekaligus. Itulah **async programming**.

[ILUSTRASI: Perbandingan pelayan restoran sinkronus (berdiri nunggu) vs asinkronus (melayani banyak meja sekaligus)]

---

## Sinkronus vs Asinkronus

| Sinkronus | Asinkronus |
|-----------|-----------|
| Satu tugas selesai dulu, baru lanjut | Bisa "pause" dan kerjain tugas lain |
| Gampang dipahami | Sedikit lebih kompleks |
| Cocok untuk CPU-heavy | Cocok untuk I/O-heavy (database, HTTP) |
| Program "block" sambil nunggu | Program tetap responsif |

Untuk web API seperti Support Desk, hampir semua operasi itu I/O: baca dari database, kirim HTTP response, tulis log. Kalau semua dikerjain sinkronus, server bakal lemot parah karena nunggu terus.

---

## async fn dan .await

Sebuah fungsi ditandai sebagai async dengan keyword `async fn`:

```rust
async fn proses_ticket(id: u32) -> String {
    // fungsi ini bisa di-pause dan dilanjutkan nanti
    format!("Ticket #{} selesai", id)
}
```

Untuk "menunggu" hasil dari async function, pakai `.await`:

```rust
async fn main() {
    let hasil = proses_ticket(1).await; // tunggu sampai selesai
    println!("{}", hasil);
}
```

`.await` artinya: "tunggu operasi ini selesai, tapi sambil nunggu, beri kesempatan program ngerjain hal lain." Ini berbeda dengan `thread::sleep` biasa yang beneran membekukan program.

---

## Future: Janji yang Belum Selesai

Waktu `async fn` dipanggil, Rust nggak langsung jalankan kodenya. Yang didapat adalah sebuah **Future**, semacam "janji" bahwa operasi itu akan selesai di masa depan.

Analoginya seperti pesan makanan online dan dapat nomor resi. Makanan belum datang, tapi ada "bukti" bahwa dia akan datang. Baru pas `.await`, kamu beneran nunggu makanan itu tiba.

Future di Rust sifatnya **lazy**: tidak dikerjakan sampai ada yang `.await` dia. Ini yang bikin Rust efisien karena kamu bisa bikin banyak "janji" dan pilih kapan mau ditunggu.

---

## Tokio: Runtime Async untuk Rust

Rust tidak punya async runtime bawaan. Go punya goroutine built-in, Node.js punya event loop bawaan. Rust sengaja tidak menyertakan ini di bahasa inti, supaya kamu bisa pilih runtime yang paling cocok untuk kebutuhan.

**Tokio** adalah async runtime paling populer untuk Rust. Dia yang bertugas menjadwalkan Future mana yang jalan sekarang, mengelola thread pool di belakang layar, dan menyediakan tools async seperti timer, channel, dan file I/O.

Tambahkan Tokio ke `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
# tokio versi terbaru saat ebook ini ditulis, Maret 2026: ~1.49
# cek https://crates.io/crates/tokio untuk versi terkini
```

`features = ["full"]` mengaktifkan semua fitur Tokio. Untuk production, kamu bisa pilih fitur spesifik agar binary lebih kecil, tapi untuk belajar, `"full"` praktis.

---

## #[tokio::main]

Fungsi `main()` di Rust bersifat sinkronus. Tidak bisa langsung pakai `async fn main()` tanpa runtime yang menjalankan event loop-nya. Solusinya adalah macro `#[tokio::main]`:

```rust
#[tokio::main]
async fn main() {
    // sekarang kamu bisa .await di sini
    println!("Support Desk API siap!");
}
```

Macro ini secara otomatis membungkus `main()` supaya Tokio runtime berjalan di belakang layar. Kamu tinggal menulis logika async.

---

## tokio::spawn: Jalankan Bersamaan

`.await` artinya "tunggu ini selesai dulu." `tokio::spawn` artinya "jalankan ini di background, kita lanjut dulu." Seperti pelayan yang simultan kasih pesanan ke dapur untuk banyak meja sekaligus, bukan satu-satu.

```rust
use tokio::time::{sleep, Duration};

async fn proses_ticket(id: u32) -> String {
    println!("Mulai proses ticket #{}...", id);
    sleep(Duration::from_millis(100)).await; // simulasi operasi I/O
    format!("Ticket #{} selesai diproses", id)
}

async fn kirim_notifikasi(user_id: u32) {
    sleep(Duration::from_millis(50)).await;
    println!("Notifikasi terkirim ke user #{}", user_id);
}

#[tokio::main]
async fn main() {
    // Sequential — satu per satu, total ~100ms
    let result = proses_ticket(1).await;
    println!("{}", result);

    // Concurrent dengan spawn — jalan bersamaan, total ~100ms (bukan 300ms!)
    let handle1 = tokio::spawn(proses_ticket(2));
    let handle2 = tokio::spawn(proses_ticket(3));
    let handle3 = tokio::spawn(kirim_notifikasi(42));

    let r1 = handle1.await.unwrap();
    let r2 = handle2.await.unwrap();
    handle3.await.unwrap();

    println!("{}", r1);
    println!("{}", r2);
}
```

`tokio::spawn` mengembalikan sebuah `JoinHandle`. Kamu `.await` handle itu nanti untuk ambil hasilnya. Kalau ada 3 ticket yang perlu diproses masing-masing 100ms, dengan `spawn` total waktu tetap ~100ms, bukan 300ms.

---

## tokio::time::sleep: Delay Async

`tokio::time::sleep` adalah versi async dari `std::thread::sleep`, dan perbedaannya penting:

- `std::thread::sleep(100ms)` **membekukan seluruh thread**. Nggak ada yang bisa jalan.
- `tokio::time::sleep(100ms).await` **hanya pause task ini**. Task lain tetap jalan.

```rust
use tokio::time::{sleep, Duration};

async fn cek_status_ticket(id: u32) {
    println!("Cek ticket #{} dimulai", id);
    sleep(Duration::from_secs(1)).await; // tunggu 1 detik, tapi nggak block thread
    println!("Ticket #{} status: open", id);
}
```

Di konteks Support Desk, ini berguna untuk menunggu response dari email server, polling status eksternal, atau rate limiting ke third-party API.

[ILUSTRASI: Diagram timeline showing 3 concurrent tasks overlapping vs 3 sequential tasks stacked]

---

## Kenapa Ini Penting untuk API Kita?

Web API hampir semuanya bekerja dengan nunggu I/O: user kirim request, kita query database (nunggu), database balas, kita proses data, kita kirim email notifikasi (nunggu SMTP server), baru kita return response ke user.

Kalau semua ini sinkronus, server hanya bisa menangani **1 request sekaligus**. Dengan async, server bisa menangani **ribuan request bersamaan** dengan resource yang sama.

Framework Elysia yang dipakai di project TypeScript bekerja dengan prinsip yang sama. Axum, web framework Rust yang populer, dibangun di atas Tokio. Pemahaman soal `async`/`.await` dan `tokio::spawn` langsung kepake ketika migrasi ke Rust nanti.

[ILUSTRASI: Diagram: User requests masuk ke server → async handler memproses bersamaan → responses keluar]

---

## Latihan

Buat project baru dan praktikkan konsep ini:

```bash
cargo new support-desk-async
cd support-desk-async
```

Tambah Tokio ke `Cargo.toml`, lalu tulis kode berikut di `src/main.rs`:

**Tantangan 1: Sequential vs Concurrent**
Jalankan `proses_ticket` untuk ticket #1, #2, #3 secara sequential. Ukur waktunya dengan `std::time::Instant::now()`. Lalu ubah jadi concurrent dengan `tokio::spawn`. Bandingkan hasilnya.

**Tantangan 2: Simulasi Support Desk**
Buat async function `assign_ticket(ticket_id: u32, agent_name: &str)` yang:
1. Print "Assigning ticket #{} to {}"
2. Sleep 200ms (simulasi update database)
3. Print "Ticket #{} assigned!"

Jalankan assignment untuk 5 ticket bersamaan dengan `tokio::spawn`.

**Tantangan 3 (Bonus):**
Tambah function `broadcast_update(message: &str)` yang sleep 50ms lalu print pesan. Jalankan broadcast bersamaan dengan beberapa ticket assignments. Amati urutan output, apakah selalu sama?

Urutan output tidak dijamin sama setiap run. Itulah concurrency: tidak ada jaminan siapa yang selesai duluan. Di bab selanjutnya dibahas cara koordinasi antar task dengan channels dan mutex.

---

Konsep fundamental yang bikin Rust unggul untuk high-performance API sudah tercakup di sini. `async`/`.await` dan Tokio adalah fondasi dari hampir semua web backend modern di ekosistem Rust.
