# Rust dari Nol sampai REST API
## Belajar Rust dengan Membangun Support Desk API

---

> **Untuk siapa ebook ini?**
> Ebook ini ditulis untuk siapa saja yang ingin belajar Rust — tidak perlu pengalaman programming sebelumnya. Kamu yang sudah coding di bahasa lain akan lebih mudah, tapi bukan syarat wajib.
>
> **Apa yang akan kamu bangun?**
> Di akhir ebook ini, kamu akan punya REST API Support Desk lengkap — autentikasi JWT, manajemen tiket, role-based access control, dan database PostgreSQL. Semua ditulis dalam Rust.

---

## Daftar Isi

### Fase 1: Dasar-Dasar Rust

| Bab | Judul |
|-----|-------|
| [01](./01-mengenal-rust.md) | Mengenal Rust |
| [02](./02-instalasi-project-pertama.md) | Instalasi dan Project Pertama |
| [03](./03-rust-vs-go.md) | Rust vs Go: Kapan Pakai Yang Mana? |
| [04](./04-variabel-tipe-data.md) | Variabel dan Tipe Data |
| [05](./05-string-di-rust.md) | String di Rust |
| [06](./06-control-flow.md) | Control Flow |
| [07](./07-fungsi.md) | Fungsi |
| [08](./08-ownership.md) | Ownership |
| [09](./09-borrowing-references.md) | Borrowing dan References |
| [10](./10-struct-method.md) | Struct dan Method |
| [11](./11-enum-pattern-matching.md) | Enum dan Pattern Matching |
| [12](./12-error-handling.md) | Error Handling |
| [13](./13-traits.md) | Traits |
| [14](./14-generics-collections.md) | Generics dan Collections |
| [15](./15-iterator-closure.md) | Iterator dan Closure |
| [16](./16-module-system.md) | Module System |
| [17](./17-async-tokio.md) | Async Rust dan Tokio |

### Fase 2: Membangun Support Desk API

| Bab | Judul |
|-----|-------|
| [18](./18-setup-axum.md) | Setup Project Axum |
| [19](./19-routing-handler.md) | Routing dan Handler |
| [20](./20-serde-json.md) | Serde: Serialisasi JSON |
| [21](./21-validasi-input.md) | Validasi Input |
| [22](./22-respons-api-standar.md) | Respons API Standar |
| [23](./23-postgresql-sqlx-setup.md) | PostgreSQL dan SQLx Setup |
| [24](./24-migrations-schema.md) | Migrations dan Schema |
| [25](./25-repository-user-ticket.md) | Repository: User dan Ticket |
| [26](./26-repository-response-dashboard.md) | Repository: Response dan Dashboard |
| [27](./27-autentikasi-register-login.md) | Autentikasi: Register dan Login |
| [28](./28-autentikasi-jwt-middleware.md) | Autentikasi: JWT dan Middleware |
| [29](./29-role-based-access-control.md) | Role-Based Access Control |
| [30](./30-service-ticket-response.md) | Service Layer: Ticket dan Response |
| [31](./31-service-user-dashboard.md) | Service Layer: User dan Dashboard |

### Fase 3: Penyempurnaan

| Bab | Judul |
|-----|-------|
| [32](./32-menyatukan-routes.md) | Menyatukan Semua Routes |
| [33](./33-testing-unit.md) | Testing: Unit Test |
| [34](./34-testing-integrasi.md) | Testing: Integration Test |
| [35](./35-penutup.md) | Penutup dan Langkah Selanjutnya |

---

## Cara Membaca Ebook Ini

Baca **dari awal ke akhir secara berurutan**. Setiap bab membangun pemahaman dari bab sebelumnya.

Kalau sudah punya pengalaman programming, boleh skip Bab 1-3. Tapi **jangan skip Bab 4-17** — konsep Rust di sana berbeda dari bahasa lain dan jadi fondasi Fase 2.

Kode di Fase 2 (Bab 18-32) bersifat **kumulatif** — kode Bab 19 menggunakan hasil Bab 18, dan seterusnya.

---

## Yang Kamu Butuhkan

- Komputer dengan koneksi internet
- Text editor (VS Code direkomendasikan)
- Semangat belajar

Tidak perlu pengalaman Rust sebelumnya. 🦀
