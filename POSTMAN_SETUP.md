# Support Desk API - Postman Setup Guide

## 📥 Cara Import Collection ke Postman

### Step 1: Buka Postman
1. Buka aplikasi Postman atau akses web.postman.co
2. Login ke akun Postman kamu (atau skip jika tidak punya)

### Step 2: Import Collection
1. Klik tombol **"Import"** di bagian atas kiri
2. Pilih tab **"File"**
3. Klik **"Upload Files"** atau drag-drop
4. Cari file: `Support_Desk_API.postman_collection.json`
5. Klik **"Import"**

### Step 3: Import Environment (Optional tapi Recommended)
1. Klik ikon **"Settings"** (⚙️) di bagian bawah kiri
2. Pilih **"Environments"** → **"Import"**
3. Upload file: `Support_Desk_Development.postman_environment.json`
4. Klik **"Import"**

### Step 4: Pilih Environment yang Aktif
1. Di kanan atas, kamu akan lihat dropdown yang berisi **"No Environment"**
2. Klik dropdown itu
3. Pilih **"Support Desk - Development"**
4. Sekarang semua variabel ({{baseUrl}}, {{token}}, dll) siap pakai

---

## 🚀 Workflow: Dari Register sampai Create Ticket

### 1. Register User Baru
1. Buka collection **"Authentication"** → **"Register User"**
2. Edit request body jika perlu (ubah name, email, password)
3. Klik **"Send"**
4. Response akan berisi user ID dan data baru

**Catatan:** Pastikan email unik dan password minimal 8 karakter

### 2. Login untuk Mendapatkan JWT Token
1. Buka **"Authentication"** → **"Login"**
2. Edit email dan password sesuai user yang baru terdaftar
3. Klik **"Send"**
4. Response akan berisi `token`
5. **Token otomatis disimpan ke variabel {{token}}** (berkat Postman test script)

### 3. Get User Profile
1. Buka **"Users"** → **"Get My Profile"**
2. Header `Authorization: Bearer {{token}}` sudah auto-diisi
3. Klik **"Send"**
4. Kamu akan lihat profil user yang baru login

### 4. Create Ticket
1. Buka **"Tickets"** → **"Create Ticket"**
2. Edit request body sesuai keinginan (subject, description, category, priority)
3. Klik **"Send"**
4. Jika berhasil, copy UUID dari response ke variable `{{ticketId}}`
5. Atau gunakan variable itu langsung di endpoint berikutnya

---

## 📋 Daftar Endpoint Lengkap

### Health & Status
- `GET /health` - Check server status

### Authentication
- `POST /auth/register` - Register akun baru
- `POST /auth/login` - Login dan dapat JWT token

### Users
- `GET /me` - Get profile user yang sedang login
- `GET /users` - List semua user
- `GET /users/{id}` - Get user by UUID
- `GET /agents` - List semua agent
- `GET /customers` - List semua customer
- `PATCH /users/{id}` - Update user (nama)
- `DELETE /users/{id}` - Delete user

### Tickets
- `POST /tickets` - Create ticket baru
- `GET /tickets` - List tickets dengan pagination/filter
- `GET /tickets/{id}` - Get ticket detail
- `PATCH /tickets/{id}` - Update ticket
- `DELETE /tickets/{id}` - Delete ticket (admin only)

### Ticket Responses
- `POST /tickets/{id}/responses` - Add response to ticket
- `GET /tickets/{id}/responses` - Get all responses

### Dashboard
- `GET /dashboard/stats` - Get dashboard statistics (admin/agent only)

---

## 🔐 Authorization

### Token di Header
Semua endpoint kecuali `/health` dan auth endpoints memerlukan JWT token di header:
```
Authorization: Bearer {{token}}
```

Postman otomatis menambahkan header ini jika kamu sudah set variable `{{token}}`.

### Automatic Token Capture
Login endpoint (`/auth/login`) punya **test script** yang otomatis:
1. Mengambil token dari response
2. Menyimpannya ke variable `{{token}}`
3. Kamu tidak perlu copy-paste manual!

---

## 🧪 Contoh Testing Workflow

### Test 1: Register & Login
```
1. Authentication → Register User (dengan email baru)
2. Check response → user berhasil dibuat
3. Authentication → Login (dengan email/password tadi)
4. Klik Send → token otomatis tersimpan
```

### Test 2: Buat Ticket
```
1. Users → Get My Profile (pastikan token valid)
2. Check response → melihat profil user
3. Tickets → Create Ticket (isi subject, description, category, priority)
4. Klik Send → ticket berhasil dibuat
5. Copy UUID dari response → simpan ke {{ticketId}}
```

### Test 3: List & Detail Ticket
```
1. Tickets → Get All Tickets
2. Klik Send → melihat list ticket yang sudah dibuat
3. Tickets → Get Ticket by ID (gunakan {{ticketId}} dari step 5 test 2)
4. Klik Send → melihat detail ticket spesifik
```

### Test 4: Update & Delete
```
1. Tickets → Update Ticket (ubah subject atau status)
2. Klik Send → ticket berhasil update
3. Tickets → Delete Ticket
4. Klik Send → ticket berhasil delete (admin only)
```

---

## ⚡ Variable Cheatsheet

Postman collection sudah punya 4 variables built-in:

| Variable | Default | Penggunaan |
|----------|---------|-----------|
| `{{baseUrl}}` | http://localhost:3000 | URL base API |
| `{{token}}` | (kosong, diisi otomatis) | JWT token dari login |
| `{{userId}}` | (manual diisi) | User ID/UUID |
| `{{ticketId}}` | (manual diisi) | Ticket ID/UUID |

### Cara Set Manual Variable:
Jika kamu perlu manual set:
1. Di Postman, klik tab **"Environment"** (sebelah Collections)
2. Pilih **"Support Desk - Development"**
3. Edit value langsung di tabel
4. Klik **"Save"** (Ctrl+S)

---

## 🐛 Troubleshooting

### Error: "Token diperlukan" (401 Unauthorized)
**Solusi:**
1. Pastikan sudah login dulu dan dapat token
2. Pastikan environment **"Support Desk - Development"** aktif
3. Pastikan header sudah ada: `Authorization: Bearer {{token}}`
4. Cek apakah token sudah expired (token berlaku 24 jam)

### Error: "validation failed" (422 Unprocessable Entity)
**Solusi:**
- Cek request body sesuai validation rules:
  - Email: format email yang valid
  - Password: minimal 8 karakter
  - Name: 2-100 karakter
  - Subject: 5-200 karakter
  - Category: general, billing, technical, atau other
  - Priority: low, medium, high, atau urgent
  - Role: customer atau agent (bukan admin di register)

### Error: "Endpoint ini hanya untuk admin atau agent" (403 Forbidden)
**Solusi:**
- Dashboard stats hanya bisa diakses admin/agent
- Jika kamu login sebagai customer, kamu tidak bisa akses endpoint tersebut
- Buat account baru dengan role "agent" atau minta admin setup

---

## 📝 Tips & Tricks

1. **Buat Folder untuk Organize Requests**
   - Klik kanan di collection → Add Folder
   - Drag-drop requests ke folder untuk organize

2. **Set Request di Favorite**
   - Klik icon star (⭐) di sebelah request name
   - Request favorit akan muncul di bagian atas

3. **Duplicate Request**
   - Klik kanan request → Duplicate
   - Guna: buat variation dari request yang sama

4. **Import Postman Script untuk Auto-Testing**
   - Di tab "Tests", tambahkan:
   ```javascript
   pm.test("Status code is 200", function() {
     pm.response.to.have.status(200);
   });
   
   pm.test("Response has success flag", function() {
     pm.expect(pm.response.json().success).to.be.true;
   });
   ```

---

## 🎯 Next Steps

Setelah berhasil import:
1. ✅ Register user baru
2. ✅ Login untuk dapat token
3. ✅ Get profile untuk verify token
4. ✅ Create ticket pertama
5. ✅ List dan lihat detail ticket
6. ✅ Update & delete ticket (jika admin)
7. ✅ Check dashboard stats (jika agent/admin)

**Server siap, Postman siap, sekarang testing time! 🚀**

---

## 📞 Support

Jika ada error atau pertanyaan:
1. Check apakah server sudah running: `cargo run`
2. Check apakah database sudah setup: `docker compose up -d`
3. Check apakah environment sudah dipilih dengan benar
4. Baca Postman error message dengan seksama - biasanya cukup jelas

Happy testing! 🎉
