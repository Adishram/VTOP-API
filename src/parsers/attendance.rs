use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AttendanceRecord {
    pub course_code: String,
    pub course_title: String,
    pub course_type: String,
    pub slot: String,
    pub faculty: String,
    pub attended: u32,
    pub total: u32,
    pub percentage: u32,
    /// Classes needed to attend (> 0) or safe to miss (< 0) to hit the 75% target.
    pub target_classes: i32,
}

pub fn parse_attendance_html(html: &str) -> Result<Vec<AttendanceRecord>, String> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("#getStudentDetails table tbody tr, table.table tbody tr, table.table tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut records = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        // New VTOP table has 15 columns:
        // [0] Sl.No  [1] Course Code  [2] Course Title  [3] Course Type
        // [4] Slot   [5] Faculty Name  [6] Attendance Type  [7] Registration Date
        // [8] Attendance Date  [9] Attended Classes  [10] Total Classes
        // [11] Attendance Percentage  [12] Alert  [13] Status  [14] View
        if cells.len() < 12 {
            continue;
        }

        let course_code = cells[1].text().collect::<String>().trim().to_string();

        // Skip rows that don't have a valid course code (e.g. the "Total Credits" row)
        if course_code.is_empty() || course_code.starts_with("Total") {
            continue;
        }

        let course_title = cells[2].text().collect::<String>().trim().to_string();
        let course_type = cells[3].text().collect::<String>().trim().to_string();
        let slot = cells[4].text().collect::<String>().trim().to_string();

        // Faculty cell has multiple <p> tags: employee ID, Name, Department
        // Extract just the name (second line) if possible
        let faculty_raw = cells[5].text().collect::<String>();
        let faculty_lines: Vec<&str> = faculty_raw.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let faculty = if faculty_lines.len() >= 2 {
            faculty_lines[1].to_string()
        } else {
            faculty_lines.first().unwrap_or(&"").to_string()
        };

        let attended: u32 = cells[9].text().collect::<String>().trim().parse().unwrap_or(0);
        let total: u32 = cells[10].text().collect::<String>().trim().parse().unwrap_or(0);
        let percentage: u32 = cells[11].text().collect::<String>().trim().parse().unwrap_or(0);

        let target_classes = classes_to_target(attended, total, 75.0);

        records.push(AttendanceRecord {
            course_code,
            course_title,
            course_type,
            slot,
            faculty,
            attended,
            total,
            percentage,
            target_classes,
        });
    }

    Ok(records)
}

/// Pure math calculator to compute classes needed/safe to miss.
/// Returns > 0 if classes need to be attended.
/// Returns < 0 if classes can be safely missed.
/// Returns 0 if perfectly on target or if division by zero.
pub fn classes_to_target(attended: u32, total: u32, target_percent: f32) -> i32 {
    if total == 0 {
        return 0;
    }

    let current = (attended as f32) / (total as f32) * 100.0;
    let t = target_percent / 100.0;

    if current < target_percent {
        // Need to attend N classes
        // (attended + N) / (total + N) >= target/100
        if t >= 1.0 {
            return 999; // impossible to reach 100% if already missed one
        }
        let n = (t * (total as f32) - (attended as f32)) / (1.0 - t);
        n.ceil() as i32
    } else {
        // Can miss M classes
        // attended / (total + M) >= target/100
        if t <= 0.0 {
            return -999;
        }
        let m = ((attended as f32) / t) - (total as f32);
        -(m.floor() as i32)
    }
}
