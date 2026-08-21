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
    let row_selector = Selector::parse("#getStudentDetails table tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut records = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        // The fixture table has 10 columns
        if cells.len() >= 9 {
            let course_code = cells[1].text().collect::<String>().trim().to_string();
            let course_title = cells[2].text().collect::<String>().trim().to_string();
            let course_type = cells[3].text().collect::<String>().trim().to_string();
            let slot = cells[4].text().collect::<String>().trim().to_string();
            let attended: u32 = cells[5].text().collect::<String>().trim().parse().unwrap_or(0);
            let total: u32 = cells[6].text().collect::<String>().trim().parse().unwrap_or(0);
            let percentage: u32 = cells[7].text().collect::<String>().trim().parse().unwrap_or(0);
            let faculty = cells[8].text().collect::<String>().trim().to_string();

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
