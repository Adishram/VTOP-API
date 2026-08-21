use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// VTOP Quirk: Fields are represented as adjacent `<td>` pairs. We must use first-match semantics.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct StudentProfile {
    pub application_number: String,
    pub student_name: String,
    pub programme: String,
    pub branch: String,
    pub school: String,
}

pub fn parse_profile_html(html: &str) -> Result<StudentProfile, String> {
    let document = Html::parse_document(html);
    let cell_selector = Selector::parse("td").unwrap();

    let mut profile = StudentProfile::default();
    let mut current_label = String::new();

    for cell in document.select(&cell_selector) {
        let text = cell.text().collect::<String>().trim().to_string();
        
        if text.is_empty() {
            continue;
        }

        // Check if the cell is bold (a label) by checking if it contains a <b> tag or if it was marked as a label earlier.
        // Actually, VTOP usually wraps labels in <b>.
        let is_label = cell.select(&Selector::parse("b").unwrap()).next().is_some();

        if is_label {
            current_label = text;
        } else if !current_label.is_empty() {
            match current_label.as_str() {
                "Application Number" if profile.application_number.is_empty() => profile.application_number = text,
                "Student Name" if profile.student_name.is_empty() => profile.student_name = text,
                "Programme" if profile.programme.is_empty() => profile.programme = text,
                "Branch" if profile.branch.is_empty() => profile.branch = text,
                "School" if profile.school.is_empty() => profile.school = text,
                _ => {}
            }
            current_label.clear();
        }
    }

    Ok(profile)
}
