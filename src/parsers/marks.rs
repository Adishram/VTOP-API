use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// VTOP Quirk: The outer table rows alternate between a summary row (Course info) and a detail row containing a nested `table` of mark components.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CourseMarks {
    pub course_code: String,
    pub course_title: String,
    pub course_type: String,
    pub slot: String,
    pub components: Vec<MarkComponent>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MarkComponent {
    pub title: String,
    pub max_marks: String,
    pub weightage_percent: String,
    pub scored: String,
    pub weightage_mark: String,
    pub average: String,
    pub status: String,
}

pub fn parse_marks_html(html: &str) -> Result<Vec<CourseMarks>, String> {
    let document = Html::parse_document(html);
    // Find the outer table rows
    let row_selector = Selector::parse("table.customTable > tbody > tr, table.customTable > tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let inner_row_selector = Selector::parse("table tr").unwrap();

    let mut courses = Vec::new();
    let mut current_course: Option<CourseMarks> = None;

    let rows: Vec<_> = document.select(&row_selector).collect();
    
    // Skip the first row (heading)
    for row in rows.into_iter().skip(1) {
        // If row has a nested table, it's the detail row for the previous course
        if row.select(&Selector::parse("table").unwrap()).next().is_some() {
            if let Some(mut course) = current_course.take() {
                // Parse nested table
                for inner_row in row.select(&inner_row_selector).skip(1) { // skip inner header
                    let cells: Vec<_> = inner_row.select(&cell_selector).collect();
                    if cells.len() >= 8 {
                        course.components.push(MarkComponent {
                            title: cells[1].text().collect::<String>().trim().to_string(),
                            max_marks: cells[2].text().collect::<String>().trim().to_string(),
                            weightage_percent: cells[3].text().collect::<String>().trim().to_string(),
                            scored: cells[5].text().collect::<String>().trim().to_string(),
                            weightage_mark: cells[6].text().collect::<String>().trim().to_string(),
                            average: cells[7].text().collect::<String>().trim().to_string(),
                            status: cells[4].text().collect::<String>().trim().to_string(),
                        });
                    }
                }
                courses.push(course);
            }
        } else {
            // It's a summary row
            let cells: Vec<_> = row.select(&cell_selector).collect();
            if cells.len() >= 8 {
                current_course = Some(CourseMarks {
                    course_code: cells[2].text().collect::<String>().trim().to_string(),
                    course_title: cells[3].text().collect::<String>().trim().to_string(),
                    course_type: cells[4].text().collect::<String>().trim().to_string(),
                    slot: cells[7].text().collect::<String>().trim().to_string(),
                    components: Vec::new(),
                });
            }
        }
    }

    Ok(courses)
}
