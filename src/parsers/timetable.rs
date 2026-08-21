use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// VTOP Quirk: Two separate tables exist. The first `div#studentDetailsList > table` lists courses.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TimetableCourse {
    pub course: String,
    pub l_t_p_j_c: String,
    pub slot_venue: String,
    pub faculty: String,
}

pub fn parse_timetable_html(html: &str) -> Result<Vec<TimetableCourse>, String> {
    let document = Html::parse_document(html);
    // Find the first list of courses table
    let row_selector = Selector::parse("#studentDetailsList table tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut courses = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        if cells.len() >= 5 {
            courses.push(TimetableCourse {
                course: cells[1].text().collect::<String>().trim().to_string(),
                l_t_p_j_c: cells[2].text().collect::<String>().trim().to_string(),
                slot_venue: cells[3].text().collect::<String>().trim().to_string(),
                faculty: cells[4].text().collect::<String>().trim().to_string(),
            });
        }
    }

    Ok(courses)
}
