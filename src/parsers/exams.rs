use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// VTOP Quirk: Exam types (e.g., "CAT - I") are header rows that span multiple columns (`colspan > 1`).
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExamSchedule {
    pub exam_type: String,
    pub course_code: String,
    pub course_title: String,
    pub slot: String,
    pub exam_date: String,
    pub exam_time: String,
    pub venue: String,
    pub seat_location: String,
    pub seat_no: String,
}

pub fn parse_exams_html(html: &str) -> Result<Vec<ExamSchedule>, String> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut exams = Vec::new();
    let mut current_exam_type = String::new();

    let rows: Vec<_> = document.select(&row_selector).collect();
    
    // Skip the first row (table headers like Course Code, Course Title, etc)
    for row in rows.into_iter().skip(1) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        
        if cells.len() == 1 && cells[0].value().attr("colspan").is_some() {
            // It's an exam type header row
            current_exam_type = cells[0].text().collect::<String>().trim().to_string();
        } else if cells.len() >= 8 {
            exams.push(ExamSchedule {
                exam_type: current_exam_type.clone(),
                course_code: cells[0].text().collect::<String>().trim().to_string(),
                course_title: cells[1].text().collect::<String>().trim().to_string(),
                slot: cells[2].text().collect::<String>().trim().to_string(),
                exam_date: cells[3].text().collect::<String>().trim().to_string(),
                exam_time: cells[4].text().collect::<String>().trim().to_string(),
                venue: cells[5].text().collect::<String>().trim().to_string(),
                seat_location: cells[6].text().collect::<String>().trim().to_string(),
                seat_no: cells[7].text().collect::<String>().trim().to_string(),
            });
        }
    }

    Ok(exams)
}
