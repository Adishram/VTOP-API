# VTOP-API

A lightning-fast, headless REST API built in Rust (`axum`) for interacting with the VIT Chennai VTOP Portal.

## Features

- **Fast & Concurrent:** Built with Rust, leveraging `axum` and `tokio` for high performance.
- **Academic Data Extraction:** Seamlessly fetches Attendance, Marks, Timetable, Grades, Faculty Profile, and Academic Calendar.
- **Smart Caching & Rate Limiting:** Prevents your accounts from getting locked by VTOP due to excessive requests.
- **Plug-and-Play:** Easily deployable using Docker and Docker Compose.

---

## 🚀 Quick Start (Docker)

The absolute easiest way to run VTOP-API is using Docker. It requires zero dependencies other than Docker and Docker Compose.

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Adishram/VTOP-API.git
   cd VTOP-API
   ```

2. **Start the API:**
   ```bash
   docker compose up -d
   ```

The API is now running locally on `http://127.0.0.1:3000`!



## 📖 API Usage Guide

Testing the API is incredibly simple. Below is a step-by-step walkthrough using `httpie` (you can also use Postman, Insomnia, or cURL).

### 💡 HTTPie Syntax Cheat Sheet
Before getting started, here's how `httpie` parses arguments:
- **JSON Body fields:** `key="value"` (e.g., `username="123"`)
- **Query Parameters:** `key==value` (e.g., `semester==CH2025261`) - Note the double equals `==`! (Alternatively, append them directly to the URL like `?semester=CH2025261`)
- **HTTP Headers:** `Header:Value` (e.g., `Authorization:"Bearer TOKEN"`)

### 1. Initiate Login

First, request a captcha image and an internal login session ID.

```bash
http POST http://127.0.0.1:3000/api/login username="YOUR_REGISTRATION_NUMBER"
```

**Expected Response:**
```json
{
  "status": "ok",
  "data": {
    "login_session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "captcha_image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA..."
  }
}
```
*Note: You can copy the base64 string into a web browser URL bar to view and solve the captcha.*

### 2. Verify Login

Submit your credentials along with the solved captcha and the `login_session_id` from Step 1.

```bash
http POST http://127.0.0.1:3000/api/login/verify \
    login_session_id="a1b2c3d4-e5f6-7890-abcd-ef1234567890" \
    username="YOUR_REGISTRATION_NUMBER" \
    password="YOUR_PASSWORD" \
    captcha="SOLVED_CAPTCHA_TEXT"
```

**Expected Response:**
```json
{
  "status": "ok",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```
*Keep this `token` handy. You will use it as a Bearer token in the `Authorization` header for all subsequent requests.*

### 3. Fetch Available Semesters

Before fetching academic data, retrieve the list of available semesters.

```bash
http GET "http://127.0.0.1:3000/api/courses" \
    Authorization:"Bearer YOUR_JWT_TOKEN"
```

Find the semester ID you want to query (e.g., `CH2025261`).

### 4. Fetch Academic Data

You can now use the JWT token and your chosen semester ID to seamlessly interact with any of the academic endpoints.

**Attendance:**
```bash
http GET "http://127.0.0.1:3000/api/attendance?semester=CH2025261" \
    Authorization:"Bearer YOUR_JWT_TOKEN"
```

**Marks:**
```bash
http GET "http://127.0.0.1:3000/api/marks?semester=CH2025261" \
    Authorization:"Bearer YOUR_JWT_TOKEN"
```

**Timetable:**
```bash
http GET "http://127.0.0.1:3000/api/timetable?semester=CH2025261" \
    Authorization:"Bearer YOUR_JWT_TOKEN"
```

**Grades:**
*(No semester ID required)*
```bash
http GET "http://127.0.0.1:3000/api/grades" \
    Authorization:"Bearer YOUR_JWT_TOKEN"
```

---

## 🔒 Security & Privacy

This proxy acts as a secure, stateless bridge.
- **No Credentials Stored:** User passwords are submitted directly to VTOP and are NEVER logged or stored.
- **In-Memory Sessions:** Authenticated sessions are held in memory and automatically garbage-collected after expiration.
- **Fast:** Subsequent requests hit cached data (when appropriate) to reduce latency and VTOP server load.

