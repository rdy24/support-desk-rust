# Bab 03: Rust vs Go: Kapan Pakai Mana?

Ada satu pertanyaan yang pasti muncul sebelum deep dive ke Rust: "Kenapa tidak Go saja?" Bab ini menjawab itu, bukan untuk provokasi "mana yang lebih baik", tapi supaya kamu bisa pilih alat yang tepat untuk pekerjaan yang tepat.

Spoiler: keduanya hebat. Tapi untuk alasan yang berbeda.

---

## Filosofi Go: Simpel, Cepat, Selesai

Bayangkan kamu buka warung makan. Kamu mau fokus masak dan layani pelanggan, bukan mikirin sistem kasir, dekorasi mewah, atau workflow dapur yang super kompleks. Kamu butuh sesuatu yang **langsung jalan dan mudah dioperasikan siapapun di tim**.

Go dibuat oleh Google dengan satu tujuan utama: **produktivitas tim besar**. Sintaksnya sengaja dibuat minimal, tidak ada banyak pilihan cara untuk melakukan hal yang sama. Ada satu cara, semua orang pakai cara itu. Filosofi Go bisa diringkas jadi: **"Get things done."**

Belajar Go dalam hitungan hari, bukan bulan. Kode Go mudah dibaca orang lain yang baru gabung tim, tooling sudah built-in dan straightforward, dan Go sangat mature untuk microservices dan backend API. Go adalah bahasa yang **pragmatis**: fokus ke hasil nyata, bukan kesempurnaan teori.

[ILUSTRASI: warung makan sederhana yang ramai pelanggan vs restoran fine-dining yang sepi — Go adalah warung yang efisien]

---

## Filosofi Rust: Aman, Terkontrol, Tanpa Kompromi

Sekarang bayangkan kamu bukan buka warung, tapi membangun jembatan. Kamu tidak bisa "nanti diperbaiki kalau ada yang roboh." Setiap material, setiap baut, setiap perhitungan harus benar **sebelum** konstruksi selesai.

Itulah Rust. Filosofinya: **"If it compiles, it works."** Kalau kode kamu berhasil dikompilasi, program itu hampir pasti tidak punya bug kelas tertentu yang berbahaya.

Rust memberikan **kontrol penuh** atas bagaimana program bekerja di level paling bawah: seberapa banyak memori dipakai, kapan dibersihkan, bagaimana data berpindah. Semua itu dikontrol **oleh kamu**, bukan oleh sistem otomatis. Filosofi Rust: **"Safety without sacrifice."** Aman, tapi tetap cepat.

Belajar Rust butuh waktu lebih panjang, dan compiler-nya cerewet menolak kode yang berpotensi berbahaya. Tapi hasilnya cocok untuk software yang tidak boleh crash: sistem embedded, game engine, browser engine, dengan performa mendekati C/C++.

---

## Perbandingan Head-to-Head

| Aspek | Go | Rust |
|---|---|---|
| Memory Management | Garbage Collector (otomatis) | Ownership System (manual tapi aman) |
| Concurrency | Goroutine (ringan, mudah) | async/await + Tokio (powerful, fleksibel) |
| Learning Curve | Rendah — minggu pertama sudah produktif | Tinggi — butuh waktu untuk "klik" |
| Performa | Sangat Baik | Sangat Baik (sedikit lebih unggul) |
| Safety | Runtime errors mungkin terjadi | Compile-time safety (error ketahuan sebelum jalan) |
| Ekosistem | Mature, banyak library siap pakai | Berkembang pesat, kualitas tinggi |
| Ukuran Binary | Sedang | Kecil (tanpa runtime tambahan) |
| Cocok untuk | Microservices, CLI tools, backend API | System programming, WebAssembly, performa kritis |

---

## Memory: GC vs Ownership

Ini perbedaan paling fundamental antara Go dan Rust.

**Go pakai Garbage Collector (GC).** GC adalah "petugas kebersihan" otomatis yang bekerja di belakang layar. Kamu pakai memori sesukanya, dan GC secara berkala membersihkan memori yang tidak lagi dipakai. Nyaman sekali, tapi ada harganya: sesekali GC "pause" sebentar untuk beresin sampah, dan ini bisa jadi masalah kalau program butuh respons super cepat dan konsisten. Seperti apartemen dengan cleaning service, kamu tidak perlu pikirin beres-beres, tapi kadang cleaning service datang di waktu yang tidak kamu mau.

**Rust pakai Ownership System.** Tidak ada GC. Rust memakai aturan kepemilikan data yang unik: setiap data punya satu "pemilik" di satu waktu. Ketika pemiliknya selesai dipakai, memorinya langsung dibebaskan, otomatis, tapi dengan aturan ketat yang dicek saat kompilasi. Seperti rumah milik sendiri, kamu yang urus semua, tapi kamu tahu persis kondisi rumahmu dan tidak ada kejutan mendadak.

[ILUSTRASI: dua apartemen — satu dengan cleaning service (Go/GC) dan satu rumah milik sendiri yang kamu urus (Rust/Ownership)]

Hasilnya? Rust tidak punya "pause" dari GC. Performa Rust lebih **konsisten dan predictable**, sangat penting untuk real-time systems seperti game atau sistem kontrol.

---

## Concurrency: Goroutine vs Async/Await

**Concurrency** artinya kemampuan menjalankan banyak hal "sekaligus", misalnya server yang melayani ribuan request di saat yang sama.

**Go punya goroutine.** Goroutine adalah "thread ringan" yang bisa kamu buat ribuan dengan mudah. Cukup tulis `go namaFungsi()` dan Go langsung jalankan itu secara paralel. Simple.

**Rust punya async/await + Tokio.** `async` dan `await` adalah kata kunci yang menandai kode yang bisa "dijeda" dan dilanjutkan nanti, cocok untuk operasi yang butuh menunggu (seperti request ke database atau API). Tokio adalah runtime yang mengelola semua operasi async ini secara efisien.

Rust lebih verbose untuk hal yang sama, tapi memberikan kontrol yang lebih granular dan performa yang bisa sangat tinggi tanpa overhead yang tidak perlu.

---

## Learning Curve: Jalan Berbatu vs Jalan Tol

Go ibarat **jalan tol**: lurus, mulus, langsung sampai. Dalam 1-2 minggu, programmer baru sudah bisa produktif. Konsepnya familiar, error message-nya jelas, dan dokumentasinya excellent.

Rust ibarat **jalan pegunungan**: ada tanjakan curam dan tikungan tajam. Tapi pemandangannya luar biasa dan kamu belajar banyak tentang cara kerja komputer yang sesungguhnya.

Borrow checker (sistem yang mengecek aturan ownership Rust) adalah "musuh pertama" semua pemula Rust. Dia akan sering menolak kode kamu dengan error yang awalnya membingungkan. Tapi lama-lama, kamu tidak hanya bisa nulis Rust, kamu juga **jadi programmer yang lebih baik** karena memahami konsep yang selama ini tersembunyi di balik abstraksi bahasa lain.

[ILUSTRASI: dua jalur pendaki — jalur landai Go yang ramai vs jalur curam Rust yang sepi tapi pemandangannya indah di atas]

---

## Kapan Pilih Go?

Go adalah pilihan tepat untuk tim besar dengan turnover tinggi karena engineer baru cepat produktif. Kalau deadline ketat, Go development lebih cepat untuk fitur-fitur standar. Go sangat mature untuk microservices dan REST API, lihat saja Docker, Kubernetes, dan Terraform, semua ditulis Go. Untuk startup yang butuh ship cepat, prototipe bisa jalan dalam hari, bukan minggu. Go juga sangat efisien untuk operasi I/O-heavy seperti banyak baca/tulis file, database, dan network karena goroutine-nya ringan.

---

## Kapan Pilih Rust?

Rust adalah pilihan yang tepat ketika performa kritis: game engine, real-time trading, high-frequency systems. Begitu juga ketika memori terbatas seperti di embedded systems, firmware di microcontroller, atau IoT devices. Untuk system programming seperti operating system, driver hardware, dan WebAssembly, Rust hampir tidak ada tandingannya. Kalau keamanan adalah prioritas utama dan tidak boleh ada memory bug, buffer overflow, atau data race, Rust memberikan jaminan yang tidak bisa diberikan bahasa lain. Lalu ada juga long-running services, yaitu server yang harus jalan berbulan-bulan tanpa restart dengan konsumsi memori yang stabil.

---

## Kenapa Kita Pilih Rust di Ebook Ini?

Jawabannya ada dua: **tantangan** dan **pemahaman fundamental**.

Kalau kita belajar Go, kita akan cepat produktif, tapi kita akan "tidak tahu yang tidak kita tahu." Banyak hal bekerja secara ajaib di belakang layar dan kita tidak perlu peduli.

Rust memaksa kita peduli. Rust memaksa kita mengerti apa itu ownership, kenapa lifetime penting, bagaimana memori benar-benar bekerja, kenapa concurrency bisa berbahaya. Konsep-konsep ini berlaku di **semua bahasa pemrograman**, belajar Rust berarti naik level sebagai programmer secara umum.

Selain itu, ekosistem Rust untuk web backend sedang tumbuh pesat. Framework web, library async, dan tools production-grade sudah matang dan dipakai oleh perusahaan besar seperti Discord, Cloudflare, dan AWS di infrastruktur kritisnya.

Belajar Rust sekarang adalah investasi jangka panjang. Lebih susah di awal, tapi reward-nya nyata.

Mulai Bab 04, fokus 100% ke Rust, dimulai dari konsep dasar sampai ownership yang terkenal itu.

---

## Latihan: Pilih Alat yang Tepat

Tidak ada kode di latihan ini, ini latihan berpikir. Jawab pertanyaan berikut untuk dirimu sendiri (atau diskusikan dengan teman):

1. **Skenario A:** Kamu diminta bangun backend API untuk aplikasi e-commerce startup. Tim kamu terdiri dari 5 developer yang sudah familiar dengan Python dan JavaScript tapi belum pernah pakai Go atau Rust. Deadline 3 bulan. **Kamu pilih Go atau Rust? Kenapa?**

2. **Skenario B:** Kamu bekerja di perusahaan yang membuat software untuk mesin medis (alat scan rumah sakit). Software ini tidak boleh crash, harus respons dalam milidetik, dan berjalan di hardware dengan RAM sangat terbatas. **Kamu pilih Go atau Rust? Kenapa?**

3. **Skenario C:** Kamu mau buat side project, aplikasi CLI untuk mengorganisir file musik. Kamu sendirian, tidak ada deadline, dan ingin belajar sesuatu yang baru. **Kamu pilih Go atau Rust? Kenapa?**

Tidak ada jawaban "salah", yang penting kamu bisa argumentasikan pilihanmu berdasarkan trade-off yang sudah dibahas di bab ini.
