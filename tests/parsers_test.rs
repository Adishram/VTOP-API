use std::fs;
use vtop_api::parsers::{
    calendar::{parse_academic_calendar_html, parse_calendar_months_html},
    courses::parse_courses_html,
    exams::parse_exams_html,
    faculty::parse_profile_html,
    grades::parse_grades_html,
    marks::parse_marks_html,
    timetable::parse_timetable_html,
};

#[test]
fn test_timetable_parser() {
    let html = fs::read_to_string("fixtures/timetable.html").unwrap();
    let records = parse_timetable_html(&html).unwrap();

    assert_eq!(records.len(), 6);
    assert_eq!(records[0].course, "BCSE302L - Database Management Systems (Theory)");
    assert_eq!(records[0].l_t_p_j_c, "3 0 0 0 3");
    assert_eq!(records[0].slot_venue, "A1+TA1 - AB1-301");
    assert_eq!(records[0].faculty, "Dr. Faculty One - SCOPE");

    assert_eq!(records[5].course, "BECE401L - Embedded Systems (Theory)");
}

#[test]
fn test_faculty_parser() {
    let html = fs::read_to_string("fixtures/faculty_profile.html").unwrap();
    let profile = parse_profile_html(&html).unwrap();

    assert_eq!(profile.application_number, "20251234567");
    assert_eq!(profile.student_name, "Demo Student");
    assert_eq!(profile.programme, "B.Tech");
    assert_eq!(profile.branch, "Computer Science and Engineering");
    assert_eq!(profile.school, "School of Computer Science and Engineering (SCOPE)");
}

#[test]
fn test_marks_parser() {
    let html = fs::read_to_string("fixtures/marks.html").unwrap();
    let courses = parse_marks_html(&html).unwrap();

    assert_eq!(courses.len(), 3);

    // Course 1
    assert_eq!(courses[0].course_code, "BCSE302L");
    assert_eq!(courses[0].components.len(), 4);
    assert_eq!(courses[0].components[0].title, "CAT - 1");
    assert_eq!(courses[0].components[0].max_marks, "50");
    assert_eq!(courses[0].components[0].scored, "42");
    
    assert_eq!(courses[0].components[3].title, "FAT");
    assert_eq!(courses[0].components[3].max_marks, "100");
    assert_eq!(courses[0].components[3].scored, "78");

    // Course 3
    assert_eq!(courses[2].course_code, "BMGT105L");
    assert_eq!(courses[2].components.len(), 2);
}

#[test]
fn test_grades_parser() {
    let html = fs::read_to_string("fixtures/grades.html").unwrap();
    let history = parse_grades_html(&html).unwrap();

    assert_eq!(history.cgpa, "8.71");
    assert_eq!(history.credits_registered, "28");
    assert_eq!(history.credits_earned, "28");

    assert_eq!(history.courses.len(), 8);
    assert_eq!(history.courses[0].course_code, "BCSE101L");
    assert_eq!(history.courses[0].grade, "A");
    
    assert_eq!(history.courses[7].course_code, "BECE201L");
    assert_eq!(history.courses[7].grade, "C");
}

#[test]
fn test_exams_parser() {
    let html = fs::read_to_string("fixtures/exam_schedule.html").unwrap();
    let exams = parse_exams_html(&html).unwrap();

    assert_eq!(exams.len(), 8);

    // First exam block (CAT - I)
    assert_eq!(exams[0].exam_type, "CAT - I (Continuous Assessment Test 1)");
    assert_eq!(exams[0].course_code, "BCSE302L");
    assert_eq!(exams[0].exam_date, "15-SEP-2025");
    
    // Second exam block (CAT - II)
    assert_eq!(exams[3].exam_type, "CAT - II (Continuous Assessment Test 2)");
    assert_eq!(exams[3].course_code, "BCSE302L");
    assert_eq!(exams[3].exam_date, "20-OCT-2025");

    // Third exam block (FAT)
    assert_eq!(exams[7].exam_type, "FAT (Final Assessment Test)");
    assert_eq!(exams[7].course_code, "BMGT105L");
    assert_eq!(exams[7].exam_date, "12-DEC-2025");
}

#[test]
fn test_calendar_parser() {
    let html = fs::read_to_string("fixtures/academic_calendar.html").unwrap();
    
    // Test month parsing
    let months = parse_calendar_months_html(&html).unwrap();
    assert_eq!(months.len(), 5);
    assert_eq!(months[0], "01-AUG-2025");
    assert_eq!(months[4], "01-DEC-2025");

    // Test grid parsing
    let days = parse_academic_calendar_html(&html).unwrap();
    assert_eq!(days.len(), 21); // 3 weeks shown in fixture

    assert_eq!(days[0].date_number, "1");
    assert_eq!(days[0].info, "Instructional Day 1");
    assert_eq!(days[0].detail, Some("(Semester Classes Begin)".to_string()));

    assert_eq!(days[5].date_number, "6");
    assert_eq!(days[5].info, "No Instructional Day");
    assert_eq!(days[5].detail, Some("(Working Saturday) MONDAY Day Order".to_string()));

    assert_eq!(days[20].date_number, "21");
    assert_eq!(days[20].info, "No Instructional Day");
    assert_eq!(days[20].detail, Some("(Sunday)".to_string()));
}

#[test]
fn test_courses_parser() {
    let html = fs::read_to_string("fixtures/course_page.html").unwrap();
    let semesters = parse_courses_html(&html).unwrap();

    assert_eq!(semesters.len(), 5);
    assert_eq!(semesters[0].id, "CH2025261");
    assert_eq!(semesters[0].name, "Fall Semester 2025-26");

    assert_eq!(semesters[4].id, "CH2024253");
    assert_eq!(semesters[4].name, "Summer Semester 2024-25");
}
