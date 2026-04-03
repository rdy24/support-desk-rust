# Bab 31: Service Layer — User dan Dashboard

Auth sudah berjalan. Ticket juga bisa dibuat dan dikelola. Sekarang saatnya bangun bagian yang mengurus **siapa saja yang ada di sistem** dan **bagaimana kondisi sistem secara keseluruhan**.

---

## Analogi: HRD dan Bagian Pelaporan

Bayangkan perusahaan dengan dua divisi. **HRD** mengurus data karyawan: siapa masuk, siapa keluar, update jabatan, lihat profil. **Pelaporan** cuma bikin laporan: berapa tiket, berapa selesai, berapa pending.

HRD nggak ikut campur laporan. Pelaporan nggak ikut campur data karyawan. Keduanya punya sumber data sendiri dan staff sendiri.

Itulah `UserService` dan `DashboardService`.

[ILUSTRASI: dua gedung kantor terpisah — satu "HRD / UserService", satu "Pelaporan / DashboardService", keduanya terhubung ke "Database"]

---

## UserService: Mengelola Data User

```rust
#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }
}
```

---

### get_me: Profil Sendiri

```rust
pub async fn get_me(&self, claims: &Claims) -> Result<User, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user id".to_string()))?;

    self.user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
}
```

`claims.sub` adalah user ID dalam JWT. Parse jadi Uuid, lalu cari di database.

---

### get_all: List dengan Optional Role Filter

```rust
pub async fn get_all(
    &self,
    role: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<(Vec<User>, i64), AppError> {
    self.user_repo.find_all(role, page, limit).await
}
```

Untuk admin: filter role opsional, bisa filter "agent" atau "customer".

---

### get_by_id dan get_all: Simple Pass-through

```rust
pub async fn get_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
    self.user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
}
```

---

### update: Update User

```rust
pub async fn update(&self, user_id: Uuid, dto: UpdateUserDto) -> Result<User, AppError> {
    // Cek apakah ada field yang diupdate
    if dto.name.is_none() && dto.role.is_none() {
        return Err(AppError::BadRequest(
            "Tidak ada field yang diupdate".to_string(),
        ));
    }

    self.user_repo
        .update(user_id, dto.name.as_deref(), dto.role.as_deref())
        .await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
}
```

`UpdateUserDto` hanya punya field yang boleh diupdate: name dan role (optional).

---

### delete: Tidak Boleh Hapus Diri Sendiri

```rust
pub async fn delete(&self, target_id: Uuid, claims: &Claims) -> Result<(), AppError> {
    // Tidak boleh menghapus diri sendiri
    if target_id.to_string() == claims.sub {
        return Err(AppError::Forbidden(
            "Tidak bisa menghapus akun sendiri".to_string(),
        ));
    }

    self.user_repo.delete(target_id).await?;
    Ok(())
}
```

Aturan bisnis: bandingkan `target_id` dengan `claims.sub`. Kalau sama, tolak.

---

## DashboardService: Statistics

```rust
#[derive(Clone)]
pub struct DashboardService {
    dashboard_repo: DashboardRepository,
}

impl DashboardService {
    pub fn new(dashboard_repo: DashboardRepository) -> Self {
        Self { dashboard_repo }
    }

    pub async fn get_stats(&self) -> Result<DashboardStats, AppError> {
        self.dashboard_repo.get_stats().await
    }
}
```

Sederhana: aggregate semua statistik dari repository.

---

## Handlers: Free Functions

Pattern yang sama dari Ch30: State extractor, role-based extractors, service calls.

```rust
pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = state.user_service.get_me(&claims).await?;
    Ok(Json(json!({
        "success": true,
        "data": user
    })))
}

pub async fn get_all_users(
    State(state): State<AppState>,
    AdminOnly(_claims): AdminOnly,
    Query(filters): Query<UserFilters>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (users, total) = state
        .user_service
        .get_all(None, filters.page.unwrap_or(1) as i64, filters.limit.unwrap_or(10) as i64)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": users,
        "total": total
    })))
}

pub async fn get_agents(
    State(state): State<AppState>,
    AdminOnly(_claims): AdminOnly,
    Query(filters): Query<UserFilters>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (agents, total) = state
        .user_service
        .get_all(Some("agent"), filters.page.unwrap_or(1) as i64, filters.limit.unwrap_or(10) as i64)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": agents,
        "total": total
    })))
}

pub async fn get_customers(
    State(state): State<AppState>,
    AdminOnly(_claims): AdminOnly,
    Query(filters): Query<UserFilters>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (customers, total) = state
        .user_service
        .get_all(Some("customer"), filters.page.unwrap_or(1) as i64, filters.limit.unwrap_or(10) as i64)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": customers,
        "total": total
    })))
}

pub async fn get_user(
    State(state): State<AppState>,
    AdminOnly(_claims): AdminOnly,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = state.user_service.get_by_id(user_id).await?;
    Ok(Json(json!({
        "success": true,
        "data": user
    })))
}

pub async fn update_user(
    State(state): State<AppState>,
    AdminOnly(_claims): AdminOnly,
    Path(user_id): Path<Uuid>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = state.user_service.update(user_id, dto).await?;
    Ok(Json(json!({
        "success": true,
        "data": user
    })))
}

pub async fn delete_user(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.user_service.delete(user_id, &claims).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_stats(
    State(state): State<AppState>,
    AdminOrAgent(_claims): AdminOrAgent,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = state.dashboard_service.get_stats().await?;
    Ok(Json(json!({
        "success": true,
        "data": stats
    })))
}
```

---

## Tabel Endpoint

| Endpoint | Method | Role | Service |
|---|---|---|---|
| `/me` | GET | AuthUser | get_me() |
| `/users` | GET | AdminOnly | get_all(None) |
| `/users/:id` | GET | AdminOnly | get_by_id() |
| `/users/:id` | PATCH | AdminOnly | update() |
| `/users/:id` | DELETE | AdminOnly | delete() |
| `/agents` | GET | AdminOnly | get_all("agent") |
| `/customers` | GET | AdminOnly | get_all("customer") |
| `/dashboard/stats` | GET | AdminOrAgent | get_stats() |

---

## Latihan

1. Test `/me` endpoint dengan valid token customer
2. Test `/users` endpoint dengan admin token
3. Test `/users/:id` DELETE dengan admin, coba delete diri sendiri (harus 403 Forbidden)
4. Test role filter: `/agents` vs `/customers` vs `/users`

---

## Hasil Akhir

### Step 1: `src/repositories/user_repository.rs` — Update find_all, add update

Modified `find_all()` signature:
```rust
pub async fn find_all(
    &self,
    role: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<(Vec<User>, i64), AppError>
```

Added `update()` method with dynamic SQL.

### Step 2: `src/dto/user_dto.rs` — Add UpdateUserDto, UserFilters

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserFilters {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
```

### Step 3: `src/services/user_service.rs` — NEW

With methods: `get_me()`, `get_all()`, `get_by_id()`, `update()`, `delete()`

### Step 4: `src/services/dashboard_service.rs` — NEW

With method: `get_stats()`

### Step 5: `src/services/mod.rs` — Update

```rust
pub mod user_service;
pub mod dashboard_service;
pub use user_service::UserService;
pub use dashboard_service::DashboardService;
```

### Step 6: `src/handlers/user_handler.rs` — NEW

7 free function handlers

### Step 7: `src/handlers/dashboard_handler.rs` — NEW

1 handler: `get_stats()`

### Step 8: `src/handlers/mod.rs` — Update

Add module declarations

### Step 9: `src/main.rs` — Update

- Add `user_service` and `dashboard_service` to AppState
- Initialize in AppState::new()
- Add all routes (7 user routes + 1 dashboard route)

✅ `cargo build` → 0 errors
