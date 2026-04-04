# Bab 01: Mengenal Rust

Sebelum nulis satu baris kode pun, perlu ada gambaran dulu soal Rust: siapa dia, kenapa diciptakan, dan apakah dia relevan buat kamu. Anggap bab ini sebagai orientasi sebelum kerja keras dimulai.

---

## Apa Itu Rust?

Ada dua jenis alat di bengkel: pisau lipat multifungsi yang praktis dibawa ke mana-mana, dan mesin bubut presisi yang butuh waktu untuk dikuasai tapi hasilnya beda kelas. Python dan JavaScript masuk kategori pertama: serbaguna, cepat dipelajari, cocok untuk banyak situasi. Rust masuk kategori kedua.

**Rust adalah bahasa pemrograman sistem** (systems programming language) yang fokus pada tiga hal sekaligus: **kecepatan, keamanan memori, dan konkurensi yang aman**.

"Bahasa pemrograman sistem" berarti kita bisa bangun hal-hal yang sangat dekat dengan cara kerja komputer, bukan hanya aplikasi di atasnya, tapi infrastrukturnya sendiri.

[ILUSTRASI: perbandingan visual tiga lapisan — hardware di bawah, sistem (Rust ada di sini) di tengah, aplikasi di atas]

---

## Kenapa Rust Diciptakan?

Rust lahir dari rasa frustrasi.

Di tahun 2006, seorang engineer Mozilla bernama Graydon Hoare lagi kesal. Lift di apartemennya rusak lagi, dan ternyata software yang ngontrol lift itu crash karena **bug memori** (memory bug). Bukan karena logikanya salah, tapi karena bahasa yang dipakai tidak punya mekanisme yang cukup untuk mencegah error memori. Dari situlah Rust mulai dikerjakan. Mozilla kemudian mendukung pengembangannya secara resmi di 2009.

### Dua masalah besar yang Rust coba selesaikan:

**1. Memory Safety (Keamanan Memori)**

Memori itu seperti meja kerja. Ketika selesai pakai sebuah alat, harus taruh balik di tempatnya. Kalau tidak, mejamu penuh sampah (memory leak), atau kamu pakai alat yang sudah dipinjam orang lain (dangling pointer, yaitu pointer yang menggantung ke memori yang sudah tidak valid).

Di C dan C++, programmer harus urus ini sendiri secara manual. Satu kesalahan kecil bisa jadi celah keamanan serius, ini penyebab sekitar 70% bug keamanan di software besar.

Rust menyelesaikan ini dengan sistem bernama **ownership** (kepemilikan) yang mengurus semuanya secara otomatis *saat kode dikompilasi* (compile time), bukan saat program berjalan. Kalau kode kamu lolos kompilasi, hampir dipastikan tidak ada bug memori.

**2. Kecepatan Tanpa Kompromi**

Python dan JavaScript punya "garbage collector", petugas kebersihan otomatis yang beresin memori sambil program jalan. Nyaman, tapi butuh sumber daya dan bisa bikin program tiba-tiba jeda.

Rust tidak punya garbage collector, tapi dia juga tidak meminta kamu urus memori manual. Ownership system mengurus semuanya tanpa overhead saat runtime. Hasilnya: kecepatan setara C dan C++, tapi jauh lebih aman.

---

## Siapa yang Pakai Rust?

Rust bukan bahasa baru yang belum teruji. Di 2025, dia sudah dipakai di tempat-tempat yang sangat kritis.

**Mozilla**: tempat Rust lahir. Mereka pakai Rust untuk komponen Firefox, terutama bagian rendering engine.

**Linux Kernel**: sejak 2022, Rust resmi diterima sebagai bahasa kedua di Linux Kernel setelah C. Linux jalan di miliaran perangkat, dari server sampai Android.

**Cloudflare**: perusahaan infrastruktur internet yang melindungi jutaan website. Mereka pakai Rust untuk proxy server yang memproses triliunan request per bulan karena performa tinggi dan penggunaan memori yang efisien.

**Discord**: platform dengan ratusan juta pengguna. Mereka migrasi layanan "Read States" dari Go ke Rust. Hasilnya: latency turun drastis dan penggunaan CPU jauh lebih stabil.

**Microsoft**: mulai menulis ulang komponen Windows dan Azure menggunakan Rust untuk alasan keamanan memori.

**Amazon Web Services (AWS)**: pakai Rust untuk Firecracker, teknologi di balik AWS Lambda dan Fargate.

Rust juga sudah bertahun-tahun berturut-turut menjadi bahasa yang paling disukai developer dalam survey Stack Overflow, rekor yang belum pernah dicapai bahasa lain.

[ILUSTRASI: peta ekosistem — logo Mozilla, Linux, Cloudflare, Discord, Microsoft, AWS tersebar, dihubungkan ke logo Rust di tengah]

---

## Rust Cocok untuk Apa?

Rust tidak diciptakan untuk menggantikan semua bahasa. Dia paling cocok di situasi-situasi ini:

**Systems Programming**: kernel OS, driver hardware, firmware. Di sinilah Rust paling bersinar karena kontrol penuh atas memori tanpa mengorbankan keamanan.

**Web Backend & API**: dengan framework seperti Axum dan Actix-web, Rust bisa bangun REST API yang sangat cepat dan aman. Ekosistem Rust untuk web backend sedang berkembang pesat.

**CLI Tools**: alat command line yang dijalankan lewat terminal. Rust menghasilkan binary tunggal yang bisa langsung dijalankan tanpa perlu install runtime seperti Node.js atau Python.

**WebAssembly (Wasm)**: teknologi yang memungkinkan kode Rust berjalan di browser, dipakai untuk aplikasi web yang butuh performa tinggi seperti editor gambar, game browser, atau tool kriptografi.

**Embedded Systems**: perangkat kecil seperti mikrokontroler. Rust cocok karena tidak butuh OS dan jejak memorinya kecil.

---

## Rust vs Python dan JavaScript: Kapan Pakai Yang Mana?

Ini bukan soal mana yang lebih baik, tapi soal alat yang tepat untuk pekerjaan yang tepat.

| Situasi | Pilihan |
|---|---|
| Bikin script otomasi cepat | Python |
| Prototipe web app dalam sehari | JavaScript/Node.js |
| API performa tinggi, jutaan request/hari | Rust |
| Machine learning, data science | Python |
| Backend yang butuh keamanan memori ketat | Rust |
| Frontend interaktif di browser | JavaScript |
| CLI tool yang bisa dibagi satu file | Rust |
| Proyek yang butuh ekosistem library luas | Python/JS |

Kalau Rust lebih cepat, kenapa tidak semua orang pakai Rust? Jawabannya jujur: **kurva belajarnya lebih curam**. Konsep seperti ownership tidak ada di bahasa lain, jadi butuh waktu untuk terbiasa. Developer Python bisa produktif dalam seminggu; developer Rust butuh beberapa bulan untuk benar-benar nyaman.

Tapi investasi itu ada hasilnya. Setelah paham Rust, kamu punya pemahaman lebih dalam tentang cara komputer bekerja, dan itu berguna di bahasa apapun yang kamu pakai.

---

## Roadmap Belajarmu

Di ebook ini, kita fokus membangun **fondasi Rust yang kuat** — mulai dari syntax dasar sampai async programming. Kamu akan menguasai:

- Variabel, tipe data, dan control flow
- Ownership, borrowing, dan lifetime — konsep unik Rust
- Error handling yang robust
- Traits dan generics untuk kode yang fleksibel
- Async/await dan Tokio untuk concurrency

Setelah fondasi kuat, kamu akan siap membangun aplikasi nyata dengan Rust.

---

## Latihan

Tidak ada coding di bab ini, tapi ada dua pertanyaan untuk direfleksikan:

1. **Konteks kamu sendiri:** Kamu lebih sering dengar Python, JavaScript, atau mungkin PHP? Coba pikirkan satu proyek yang pernah kamu buat atau lihat, kira-kira masalah apa yang muncul kalau proyek itu harus handle jutaan pengguna sekaligus?

2. **Tentang trade-off:** Dari tabel perbandingan di atas, coba identifikasi satu situasi di mana kamu *tidak* akan pilih Rust, dan jelaskan alasannya ke dirimu sendiri dengan kata-katamu sendiri.

Tidak perlu tulis jawaban di mana-mana. Cukup dipikir sebentar. Latihan refleksi ini membantu otak menyimpan konteks sebelum masuk ke materi teknis.

---

*Selanjutnya → [Bab 02: Instalasi dan Project Pertama](./02-instalasi-project-pertama.md)*
