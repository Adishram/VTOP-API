# VTOP API Backend 🎓

Welcome to the VTOP API project! This backend is a fast, async Rust service designed to scrape the VIT Chennai VTOP student portal on behalf of a user and expose their academic data as a clean, structured JSON API. 

It is designed to be the foundational backend for custom student-built apps (like React Native mobile apps or Next.js dashboards).

## ✨ Features
- **Stateless Authentication**: Uses JWTs. User credentials are *never* stored on disk, logged, or saved. 
- **Aggressive Caching**: Short-lived (3 min) memory caches prevent your frontend hot-reloads from hammering VTOP.
- **Strict Throttling**: Parallel requests to VTOP are serialized and spaced out by 500ms to mimic human clicking and prevent account bans.
- **Resilient**: Auto-retries timeouts and cleanly detects VTOP session expirations, returning actionable error codes (`VTOP_SESSION_EXPIRED`) instead of crashing.

---

## 🚀 Getting Started

### Prerequisites
- Rust (Cargo) installed.
- Access to the VIT Chennai VTOP portal (you need a valid Registration Number and Password).

### 1. Environment Setup
Create a `.env` file in the root directory (you can copy `.env.example` if it exists):
```env
# The port the API will run on
PORT=8080
# The upstream VTOP portal URL
VTOP_BASE_URL=https://vtop.vitchennai.ac.in/vtop
# Note: You can override VTOP_BASE_URL for testing against mock servers.
```

### 2. Run the Server
```bash
cargo run
```
You should see: `Listening on 0.0.0.0:8080`.

---

## 📖 API Documentation & Testing

This repository includes two ways to explore the API:
1. **OpenAPI Spec**: Open `openapi.yaml` in Swagger UI, Postman, or any OpenAPI viewer for the full schema.
2. **VS Code HTTP Client**: Open `vtop-api.http` using the **REST Client** extension in VS Code. This allows you to execute the login flow and test endpoints directly from your editor.

### The Standard Envelope
Every response from this API guarantees a consistent JSON envelope:
```json
{
  "status": "ok",      // "ok" or "error"
  "data": { ... },     // The requested payload (null if error)
  "error": null        // Error message or code (e.g. "VTOP_SESSION_EXPIRED")
}
```

---

## 🔐 The Login Flow (How to Authenticate)

Because VTOP requires solving a CAPTCHA to log in, authentication is a two-step process:

### Step 1: Initiate Login
Fetch the CAPTCHA image and get a temporary login session ID.
```bash
curl -X POST http://localhost:8080/api/login
```
**Response**:
```json
{
  "status": "ok",
  "data": {
    "login_session_id": "c7a8b9f1-...",
    "captcha_base64": "data:image/png;base64,iVBORw0KGgo..."
  },
  "error": null
}
```

### Step 2: Verify Credentials
Submit your actual credentials along with the solved CAPTCHA text.
```bash
curl -X POST http://localhost:8080/api/login/verify \
  -H "Content-Type: application/json" \
  -d '{
    "login_session_id": "YOUR_LOGIN_SESSION_ID",
    "username": "20BCE0000",
    "password": "YOUR_PASSWORD",
    "captcha_text": "Q8B9A"
  }'
```
**Response**:
```json
{
  "status": "ok",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  },
  "error": null
}
```
*Save this `token`. You must pass it as a Bearer token in the `Authorization` header for all subsequent requests.*

---

## 📚 Academic Endpoints (Authenticated)

For all the endpoints below, you must include the header:
`Authorization: Bearer YOUR_JWT_TOKEN`

Some endpoints take an optional `?semester=CH2025261` query parameter. If omitted, it defaults to the current active semester.

### 1. Attendance
Computes your attended/total classes and tells you how many you can miss to stay above 75%.
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/attendance
```

### 2. Timetable
Returns your registered courses and weekly schedule slots.
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/timetable
```

### 3. Marks
Returns detailed component marks (CAT 1, CAT 2, FAT, Assignments).
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/marks
```

### 4. Exam Schedule
Returns exam dates, reporting times, and seating venues.
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/exams
```

### 5. Grades (CGPA)
Returns your overall CGPA and a history of letter grades across all semesters.
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/grades
```

### 6. Academic Calendar
Returns instructional and non-instructional days for the semester.
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/calendar
```

### 7. Courses / Semesters
Returns the list of available semesters you can query.
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/courses
```

### 8. Faculty / Student Profile
Returns your basic profile and faculty advisor information.
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/faculty
```

---

## 🛡️ Important Notes for Frontend Devs
- **If you receive `{"status": "error", "error": "VTOP_SESSION_EXPIRED"}`**: VTOP has killed your upstream session (usually happens after 15 minutes of inactivity or randomly on VTOP's side). You must redirect the user back to the login screen and get a new token. Do not just blindly retry!
- **Caching**: The backend caches data for 3 minutes. If a student's attendance just updated, they might need to wait 3 minutes before it reflects via this API.
