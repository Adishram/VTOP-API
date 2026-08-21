# VTOP HTML Fixtures

Ground-truth HTML files representing each page type served by VIT Chennai's
VTOP portal. Every parser is written and tested against these fixtures.

## Source of truth

These fixtures are **structurally derived from real VTOP HTML** by studying the
[VtopMCP](https://github.com/Vikranth-jagdish/VtopMCP) project — a working
VIT Chennai scraper that uses the exact same endpoints and HTML selectors our
Rust parsers will target. All data is **synthetic** (no real student info).

## Fixture → VTOP endpoint mapping

| Fixture                                             | VTOP Endpoint                                           | HTTP   | Description                              |
| --------------------------------------------------- | ------------------------------------------------------- | ------ | ---------------------------------------- |
| [login_page.html](login_page.html)                  | `GET /vtop/login`                                       | GET    | Portal selector (sets JSESSIONID cookie) |
| [captcha_login_form.html](captcha_login_form.html)  | `POST /vtop/prelogin/setup` (flag=VTOP)                 | POST   | Student login form with captcha image    |
| [dashboard.html](dashboard.html)                    | `POST /vtop/login` (success) or `GET /vtop/content`     | POST   | Post-login page with `#authorizedIDX`    |
| [attendance.html](attendance.html)                  | `POST /vtop/processViewStudentAttendance`                | POST   | Attendance table (`#getStudentDetails`)  |
| [timetable.html](timetable.html)                    | `POST /vtop/processViewTimeTable`                       | POST   | Course list + weekly grid                |
| [marks.html](marks.html)                            | `POST /vtop/examinations/doStudentMarkView`              | POST   | Marks with nested component tables       |
| [exam_schedule.html](exam_schedule.html)             | `POST /vtop/examinations/doSearchExamScheduleForStudent` | POST   | Exam schedule with colspan type headers  |
| [grades.html](grades.html)                          | `POST /vtop/examinations/examGradeView/StudentGradeHistory` | POST | Grade history + CGPA + curriculum        |
| [academic_calendar.html](academic_calendar.html)    | `POST /vtop/getListForSemester` + `processViewCalendar`  | POST   | Month buttons + day table                |
| [course_page.html](course_page.html)                | `POST /vtop/academics/common/StudentTimeTableChn`        | POST   | Semester selector (`#semesterSubId`)     |
| [faculty_profile.html](faculty_profile.html)        | `POST /vtop/studentsRecord/StudentProfileAllView`        | POST   | Student profile + faculty advisor        |

## Login flow (3-step)

```
1. GET  /vtop/login                    → JSESSIONID cookie + _csrf token
2. POST /vtop/prelogin/setup           → captcha_login_form.html (base64 captcha image)
   body: flag=VTOP, _csrf=<token>
3. POST /vtop/login                    → dashboard.html (authorizedIDX + rotated _csrf)
   body: username, password, captchaStr, gResponse, _csrf=<token>
```

## Authenticated request pattern

Every subsequent POST requires these form params:
- `authorizedID` = value from `#authorizedIDX` hidden input
- `_csrf` = current CSRF token (rotated on login)
- Plus endpoint-specific params (e.g. `semesterSubId`)

## Key CSS selectors / element IDs

| Selector                              | Page             | What it contains                                    |
| ------------------------------------- | ---------------- | --------------------------------------------------- |
| `#captchaBlock img[src^="data:image"]`| captcha form     | Inline base64 captcha image                         |
| `#authorizedIDX`                      | dashboard        | Hidden input with auth token for all POST requests  |
| `meta[name="_csrf"]` / `input[name="_csrf"]` | all pages | CSRF token                                          |
| `#semesterSubId option`               | course page      | Semester dropdown                                   |
| `#getStudentDetails`                  | attendance       | Attendance table                                    |
| `#studentDetailsList table`           | timetable        | Course list table                                   |
| `#fixedTableContainer table`          | marks            | Marks outer table                                   |
| `tr.tableContent`                     | grades           | Grade rows (effective grades + curriculum)           |
| `table.calendar-table` / `#list-wrapper table` | calendar | Calendar day table                           |
| `input[name="regno"]`                 | profile          | Registration number                                 |

## Credential safety

- All registration numbers replaced with `REDACTED_REG_NO`
- All names are synthetic (`Demo Student`, `Dr. Faculty One`, etc.)
- All CSRF tokens are dummy UUIDs
- No real session cookies or `JSESSIONID` values
