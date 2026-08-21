use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// VTOP Quirk: The calendar grid maps days to nested `<span>` elements.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CalendarDay {
    pub date_number: String,
    pub info: String,
    pub detail: Option<String>,
}

pub fn parse_calendar_months_html(html: &str) -> Result<Vec<String>, String> {
    let document = Html::parse_document(html);
    let button_selector = Selector::parse("#getListForSemester button").unwrap();

    let mut months = Vec::new();

    for button in document.select(&button_selector) {
        if let Some(onclick) = button.value().attr("onclick") {
            // onclick="processViewCalendar('getListForSemester','01-AUG-2025','CH2025261','COMB')"
            // Extract '01-AUG-2025'
            let parts: Vec<&str> = onclick.split('\'').collect();
            if parts.len() >= 4 {
                months.push(parts[3].to_string());
            }
        }
    }

    Ok(months)
}

pub fn parse_academic_calendar_html(html: &str) -> Result<Vec<CalendarDay>, String> {
    let document = Html::parse_document(html);
    let cell_selector = Selector::parse(".calendar-table td").unwrap();
    let span_selector = Selector::parse("span").unwrap();

    let mut days = Vec::new();

    for cell in document.select(&cell_selector) {
        let spans: Vec<_> = cell.select(&span_selector).collect();
        if !spans.is_empty() {
            let date_number = spans[0].text().collect::<String>().trim().to_string();
            
            if date_number.is_empty() {
                continue;
            }

            let info = if spans.len() > 1 {
                spans[1].text().collect::<String>().trim().to_string()
            } else {
                String::new()
            };

            let mut detail = String::new();
            for span in spans.iter().skip(2) {
                let text = span.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    if !detail.is_empty() {
                        detail.push(' ');
                    }
                    detail.push_str(&text);
                }
            }

            let detail_opt = if detail.is_empty() { None } else { Some(detail) };

            days.push(CalendarDay {
                date_number,
                info,
                detail: detail_opt,
            });
        }
    }

    Ok(days)
}
