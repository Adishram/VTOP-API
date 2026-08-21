use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// VTOP Quirk: The "Effective Grades" table groups rows by semester visually but structurally they are flat `<tr class="tableContent">` rows.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GradeHistory {
    pub cgpa: String,
    pub credits_registered: String,
    pub credits_earned: String,
    pub courses: Vec<CourseGrade>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CourseGrade {
    pub course_code: String,
    pub title: String,
    pub course_type: String,
    pub credits: String,
    pub grade: String,
    pub exam_month: String,
}

pub fn parse_grades_html(html: &str) -> Result<GradeHistory, String> {
    let document = Html::parse_document(html);
    
    // Parse Effective Grades
    let row_selector = Selector::parse("tr.tableContent").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    
    let mut courses = Vec::new();
    
    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        // The first table (grades) has 7 columns
        if cells.len() == 7 {
            courses.push(CourseGrade {
                course_code: cells[1].text().collect::<String>().trim().to_string(),
                title: cells[2].text().collect::<String>().trim().to_string(),
                course_type: cells[3].text().collect::<String>().trim().to_string(),
                credits: cells[4].text().collect::<String>().trim().to_string(),
                grade: cells[5].text().collect::<String>().trim().to_string(),
                exam_month: cells[6].text().collect::<String>().trim().to_string(),
            });
        }
    }

    // Parse CGPA Summary (which is in a table whose second row contains the data)
    // The fixture has a table with "Credits Registered" etc.
    let cgpa_table_selector = Selector::parse("table").unwrap();
    
    let mut credits_registered = String::new();
    let mut credits_earned = String::new();
    let mut cgpa = String::new();

    for table in document.select(&cgpa_table_selector) {
        let rows: Vec<_> = table.select(&Selector::parse("tr").unwrap()).collect();
        if rows.len() >= 2 {
            let header_cells: Vec<_> = rows[0].select(&cell_selector).collect();
            if header_cells.get(0).map_or(false, |c| c.text().collect::<String>().contains("Credits Registered")) {
                let data_cells: Vec<_> = rows[1].select(&cell_selector).collect();
                if data_cells.len() >= 3 {
                    credits_registered = data_cells[0].text().collect::<String>().trim().to_string();
                    credits_earned = data_cells[1].text().collect::<String>().trim().to_string();
                    cgpa = data_cells[2].text().collect::<String>().trim().to_string();
                }
                break;
            }
        }
    }

    Ok(GradeHistory {
        cgpa,
        credits_registered,
        credits_earned,
        courses,
    })
}
