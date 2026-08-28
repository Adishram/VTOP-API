use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TimetableCourse {
    pub course_code: String,
    pub course_title: String,
    pub course_type: String,
    pub slot: String,
    pub venue: String,
    pub faculty: String,
}

pub fn parse_timetable_html(html: &str) -> Result<Vec<TimetableCourse>, String> {
    let document = Html::parse_document(html);
    let cell_selector = Selector::parse("td").unwrap();

    let mut courses = Vec::new();

    let selectors = [
        "table.table tbody tr",
        "table.table tr",
        "div#studentDetailsList table:first-of-type tbody tr",
        "#studentDetailsList table tbody tr",
        "table.customTable tbody tr",
        "div#studentDetailsList tr",
    ];

    for selector_str in &selectors {
        let row_selector = Selector::parse(selector_str).unwrap();
        let rows: Vec<_> = document.select(&row_selector).collect();
        
        if rows.is_empty() {
            continue;
        }

        for row in &rows {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            
            // Skip rows that don't have enough columns (headers, time slots, etc.)
            if cells.len() < 9 {
                continue;
            }
            
            // Extract text from the first cell to check if it's a serial number
            let first_cell = cells[0].text().collect::<String>().trim().to_string();
            if first_cell.parse::<u32>().is_err() {
                continue;
            }

            // Real HTML has Course in cells[2] with format "BCSE417L - Machine vision( Theory Only )"
            let full_course = cells[2].text().collect::<String>();
            let mut course_parts = full_course.splitn(2, '-');
            let course_code = course_parts.next().unwrap_or("").trim().to_string();
            let mut course_title = course_parts.next().unwrap_or("").trim().to_string();
            
            let course_type = if let Some(start) = course_title.find('(') {
                let end = course_title.rfind(')').unwrap_or(course_title.len());
                let c_type = course_title[start + 1..end].trim().to_string();
                course_title = course_title[..start].trim().to_string();
                c_type
            } else {
                String::new()
            };
            
            // cells[7] is Slot/ Venue
            let slot_venue_full = cells[7].text().collect::<String>();
            let mut sv_parts = slot_venue_full.splitn(2, '-');
            let slot = sv_parts.next().unwrap_or("").trim().to_string();
            let venue = sv_parts.next().unwrap_or("").trim().to_string();
            
            // cells[8] is Faculty
            let faculty_raw = cells[8].text().collect::<String>();
            let faculty = faculty_raw.split_whitespace().collect::<Vec<_>>().join(" ");
            
            courses.push(TimetableCourse {
                course_code,
                course_title,
                course_type,
                slot,
                venue,
                faculty,
            });
        }

        // If we found courses with this selector, stop trying others
        if !courses.is_empty() {
            break;
        }
    }

    Ok(courses)
}
