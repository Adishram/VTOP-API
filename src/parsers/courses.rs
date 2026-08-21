use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// VTOP Quirk: This page renders a `<select id="semesterSubId">` containing the list of available semesters.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Semester {
    pub id: String,
    pub name: String,
}

pub fn parse_courses_html(html: &str) -> Result<Vec<Semester>, String> {
    let document = Html::parse_document(html);
    let option_selector = Selector::parse("select#semesterSubId option").unwrap();

    let mut semesters = Vec::new();

    for option in document.select(&option_selector) {
        if let Some(id) = option.value().attr("value") {
            let name = option.text().collect::<String>().trim().to_string();
            // Skip the "--Select Semester--" placeholder which has empty value
            if !id.is_empty() {
                semesters.push(Semester {
                    id: id.to_string(),
                    name,
                });
            }
        }
    }

    Ok(semesters)
}
