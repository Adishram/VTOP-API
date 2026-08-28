use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::extractors::AuthContext;
use crate::parsers::attendance::{parse_attendance_html, AttendanceRecord};
use crate::response::{ApiError, ApiResponse};
use crate::state::AppState;
use crate::vtop_client;

// ──────────────────────────────────────────────
// Academic data handlers
// ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SemesterQuery {
    pub semester: Option<String>,
}

/// GET /api/attendance
pub async fn attendance(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<SemesterQuery>,
) -> Result<Json<ApiResponse<Vec<AttendanceRecord>>>, ApiError> {
    tracing::info!(endpoint = "GET /api/attendance", "Fetching attendance");
    let semester = query.semester.unwrap_or_else(|| "CH2025261".to_string());
    
    // Check Cache
    let cache_key = format!("{}:attendance:{}", auth.authorized_id, semester);
    if let Some(cached) = state.cache.get(&cache_key).await {
        tracing::debug!("Cache hit for {}", cache_key);
        let records: Vec<AttendanceRecord> = serde_json::from_value(cached)
            .map_err(|e| ApiError::internal(format!("Cache deserialize error: {}", e)))?;
        return Ok(Json(ApiResponse::ok(records)));
    }

    // Step 1: Hit the menu endpoint to initialize the session state for Attendance
    let menu_url = format!("{}/academics/common/StudentAttendance", state.config.vtop_base_url);
    let _ = vtop_client::fetch_html(&auth, &menu_url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;

    // Step 2: Fetch the actual data
    let url = format!("{}/processViewStudentAttendance", state.config.vtop_base_url);
    let html = vtop_client::fetch_html(&auth, &url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("semesterSubId", semester.as_str()),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;

    let records = parse_attendance_html(&html)
        .map_err(|e| ApiError::internal(format!("Failed to parse attendance HTML: {}", e)))?;

    // Store in cache
    let val = serde_json::to_value(&records).unwrap();
    state.cache.insert(cache_key, val).await;

    Ok(Json(ApiResponse::ok(records)))
}

/// GET /api/timetable
pub async fn timetable(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<SemesterQuery>,
) -> Result<Json<ApiResponse<Vec<crate::parsers::timetable::TimetableCourse>>>, ApiError> {
    tracing::info!(endpoint = "GET /api/timetable", "Fetching timetable");
    let semester = query.semester.unwrap_or_else(|| "CH2025261".to_string());
    
    let cache_key = format!("{}:timetable:{}", auth.authorized_id, semester);
    if let Some(cached) = state.cache.get(&cache_key).await {
        let records = serde_json::from_value(cached).unwrap();
        return Ok(Json(ApiResponse::ok(records)));
    }

    // Step 1: Hit the menu endpoint to initialize the session state for Timetable
    let menu_url = format!("{}/academics/common/StudentTimeTableChn", state.config.vtop_base_url);
    let _ = vtop_client::fetch_html(&auth, &menu_url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;

    // Step 2: Fetch the actual data
    let url = format!("{}/processViewTimeTable", state.config.vtop_base_url);
    let html = vtop_client::fetch_html(&auth, &url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("semesterSubId", semester.as_str()),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    let records = crate::parsers::timetable::parse_timetable_html(&html)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state.cache.insert(cache_key, serde_json::to_value(&records).unwrap()).await;
    Ok(Json(ApiResponse::ok(records)))
}

/// GET /api/marks
pub async fn marks(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<SemesterQuery>,
) -> Result<Json<ApiResponse<Vec<crate::parsers::marks::CourseMarks>>>, ApiError> {
    tracing::info!(endpoint = "GET /api/marks", "Fetching marks");
    let semester = query.semester.unwrap_or_else(|| "CH2025261".to_string());
    
    let cache_key = format!("{}:marks:{}", auth.authorized_id, semester);
    if let Some(cached) = state.cache.get(&cache_key).await {
        let records = serde_json::from_value(cached).unwrap();
        return Ok(Json(ApiResponse::ok(records)));
    }

    // Step 1: Hit the menu endpoint to initialize the session state for Marks
    let menu_url = format!("{}/examinations/StudentMarkView", state.config.vtop_base_url);
    let _ = vtop_client::fetch_html(&auth, &menu_url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;

    // Step 2: Fetch the actual data
    let url = format!("{}/examinations/doStudentMarkView", state.config.vtop_base_url);
    let html = vtop_client::fetch_html(&auth, &url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("semesterSubId", semester.as_str()),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    let records = crate::parsers::marks::parse_marks_html(&html)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state.cache.insert(cache_key, serde_json::to_value(&records).unwrap()).await;
    Ok(Json(ApiResponse::ok(records)))
}

/// GET /api/exams
pub async fn exams(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<SemesterQuery>,
) -> Result<Json<ApiResponse<Vec<crate::parsers::exams::ExamSchedule>>>, ApiError> {
    tracing::info!(endpoint = "GET /api/exams", "Fetching exam schedule");
    let semester = query.semester.unwrap_or_else(|| "CH2025261".to_string());
    
    let cache_key = format!("{}:exams:{}", auth.authorized_id, semester);
    if let Some(cached) = state.cache.get(&cache_key).await {
        let records = serde_json::from_value(cached).unwrap();
        return Ok(Json(ApiResponse::ok(records)));
    }

    // Step 1: Hit the menu endpoint to initialize the session state for Exams
    let menu_url = format!("{}/examinations/ExamScheduleForStudent", state.config.vtop_base_url);
    let _ = vtop_client::fetch_html(&auth, &menu_url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;

    // Step 2: Fetch the actual data
    let url = format!("{}/examinations/doSearchExamScheduleForStudent", state.config.vtop_base_url);
    let html = vtop_client::fetch_html(&auth, &url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("semesterSubId", semester.as_str()),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    let records = crate::parsers::exams::parse_exams_html(&html)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state.cache.insert(cache_key, serde_json::to_value(&records).unwrap()).await;
    Ok(Json(ApiResponse::ok(records)))
}

/// GET /api/grades
pub async fn grades(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<crate::parsers::grades::GradeHistory>>, ApiError> {
    tracing::info!(endpoint = "GET /api/grades", "Fetching grades");
    
    let cache_key = format!("{}:grades", auth.authorized_id);
    if let Some(cached) = state.cache.get(&cache_key).await {
        let records = serde_json::from_value(cached).unwrap();
        return Ok(Json(ApiResponse::ok(records)));
    }

    let url = format!("{}/examinations/examGradeView/StudentGradeHistory", state.config.vtop_base_url);
    let html = vtop_client::fetch_html(&auth, &url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    let records = crate::parsers::grades::parse_grades_html(&html)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state.cache.insert(cache_key, serde_json::to_value(&records).unwrap()).await;
    Ok(Json(ApiResponse::ok(records)))
}

/// GET /api/calendar
pub async fn calendar(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Vec<crate::parsers::calendar::CalendarDay>>>, ApiError> {
    tracing::info!(endpoint = "GET /api/calendar", "Fetching calendar for default month");
    
    let cache_key = format!("{}:calendar", auth.authorized_id);
    if let Some(cached) = state.cache.get(&cache_key).await {
        let records = serde_json::from_value(cached).unwrap();
        return Ok(Json(ApiResponse::ok(records)));
    }

    // Step 1: CalendarPreview
    let url1 = format!("{}/academics/common/CalendarPreview", state.config.vtop_base_url);
    vtop_client::fetch_html(&auth, &url1, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    // Step 2: getListForSemester
    let url2 = format!("{}/getListForSemester", state.config.vtop_base_url);
    let html2 = vtop_client::fetch_html(&auth, &url2, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("paramReturnId", "getListForSemester"),
        ("semSubId", ""),
        ("classGroupId", "COMB"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    let months = crate::parsers::calendar::parse_calendar_months_html(&html2)
        .map_err(|e| ApiError::internal(e.to_string()))?;
        
    let mut all_days = Vec::new();
    
    // Step 3: processViewCalendar for each month
    let url3 = format!("{}/processViewCalendar", state.config.vtop_base_url);
    for month in months {
        let html3 = vtop_client::fetch_html(&auth, &url3, &[
            ("authorizedID", auth.authorized_id.as_str()),
            ("calDate", month.as_str()),
            ("semSubId", ""),
            ("classGroupId", "COMB"),
            ("_csrf", auth.csrf_token.as_str()),
        ]).await?;
            
        let days = crate::parsers::calendar::parse_academic_calendar_html(&html3)
            .map_err(|e| ApiError::internal(e.to_string()))?;
            
        all_days.extend(days);
    }
    
    state.cache.insert(cache_key, serde_json::to_value(&all_days).unwrap()).await;
    Ok(Json(ApiResponse::ok(all_days)))
}

/// GET /api/courses
pub async fn courses(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Vec<crate::parsers::courses::Semester>>>, ApiError> {
    tracing::info!(endpoint = "GET /api/courses", "Fetching courses/semesters list");
    
    let cache_key = format!("{}:courses", auth.authorized_id);
    if let Some(cached) = state.cache.get(&cache_key).await {
        let records = serde_json::from_value(cached).unwrap();
        return Ok(Json(ApiResponse::ok(records)));
    }

    let url = format!("{}/academics/common/StudentTimeTableChn", state.config.vtop_base_url);
    let html = vtop_client::fetch_html(&auth, &url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    let records = crate::parsers::courses::parse_courses_html(&html)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state.cache.insert(cache_key, serde_json::to_value(&records).unwrap()).await;
    Ok(Json(ApiResponse::ok(records)))
}

/// GET /api/faculty
pub async fn faculty(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<crate::parsers::faculty::StudentProfile>>, ApiError> {
    tracing::info!(endpoint = "GET /api/faculty", "Fetching faculty/profile");
    
    let cache_key = format!("{}:faculty", auth.authorized_id);
    if let Some(cached) = state.cache.get(&cache_key).await {
        let profile = serde_json::from_value(cached).unwrap();
        return Ok(Json(ApiResponse::ok(profile)));
    }

    let url = format!("{}/studentsRecord/StudentProfileAllView", state.config.vtop_base_url);
    let html = vtop_client::fetch_html(&auth, &url, &[
        ("authorizedID", auth.authorized_id.as_str()),
        ("verifyMenu", "true"),
        ("_csrf", auth.csrf_token.as_str()),
    ]).await?;
        
    let profile = crate::parsers::faculty::parse_profile_html(&html)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    state.cache.insert(cache_key, serde_json::to_value(&profile).unwrap()).await;
    Ok(Json(ApiResponse::ok(profile)))
}
