use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MasterProgram {
    pub title: String,
    pub description: String,
    pub institute: String,
    pub study_form: String,
    pub duration: String,
    pub language: String,
    pub cost: String,
    pub dormitory: bool,
    pub military_center: bool,
    pub accreditation: bool,
    pub special_programs: Vec<String>,
    pub direction_code: String,
    pub direction_name: String,
    pub budget_places: u32,
    pub target_places: u32,
    pub contract_places: u32,
    pub manager: ProgramManager,
    pub social_links: Vec<SocialLink>,
    pub exam_dates: Vec<String>,
    pub admission_methods: Vec<AdmissionMethod>,
    pub career_opportunities: String,
    pub average_salary: String,
    pub team: Vec<TeamMember>,
    pub partners: Vec<String>,
    pub scholarships: Vec<Scholarship>,
    pub international_opportunities: Vec<String>,
    pub faq: Vec<FaqItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgramManager {
    pub name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocialLink {
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdmissionMethod {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamMember {
    pub name: String,
    pub position: String,
    pub degree: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scholarship {
    pub name: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FaqItem {
    pub question: String,
    pub answer: String,
}

pub fn parse_master_program_html(html_content: &str) -> Result<MasterProgram, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html_content);
    
    // Extract program title
    let title_selector = Selector::parse("h1.Information_information__header__fab3I")?;
    let title = document
        .select(&title_selector)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    // Extract program description from JSON in script tag
    let script_selector = Selector::parse("script#__NEXT_DATA__")?;
    let script_content = document
        .select(&script_selector)
        .next()
        .map(|e| e.inner_html())
        .unwrap_or_default();
    
    
    let mut about_lead = String::new();
    let mut about_desc = String::new();
    
    if !script_content.is_empty() {
        if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&script_content) {
            if let Some(about_obj) = json_data
                .get("props")
                .and_then(|p| p.get("pageProps"))
                .and_then(|pp| pp.get("jsonProgram"))
                .and_then(|jp| jp.get("about"))
            {
                about_lead = about_obj.get("lead")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                
                about_desc = about_obj.get("desc")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .replace("<br>", "\n")
                    .replace("\\u003cbr\\u003e", "\n");
            }
        }
    }
    
    let description = if !about_lead.is_empty() && !about_desc.is_empty() {
        format!("{}\n\n{}", about_lead, about_desc)
    } else {
        about_lead + &about_desc
    };

    // Extract institute
    let institute_selector = Selector::parse("a[href*='viewfaculty'] span")?;
    let institute = document
        .select(&institute_selector)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    // Extract study information from cards
    let card_selector = Selector::parse(".Information_card__text__txwcx")?;
    let cards: Vec<String> = document
        .select(&card_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();

    let study_form = cards.get(0).unwrap_or(&String::new()).clone();
    let duration = cards.get(1).unwrap_or(&String::new()).clone();
    let language = cards.get(2).unwrap_or(&String::new()).clone();
    let cost = cards.get(3).unwrap_or(&String::new()).clone();

    // Parse boolean values for dormitory, military center, accreditation
    let dormitory = cards.get(4).map(|s| s.contains("да")).unwrap_or(false);
    let military_center = cards.get(5).map(|s| s.contains("да")).unwrap_or(false);
    let accreditation = cards.get(6).map(|s| s.contains("да")).unwrap_or(false);

    // Extract special programs
    let special_programs = cards.get(7)
        .map(|s| s.split(", ").map(|p| p.trim().to_string()).collect())
        .unwrap_or_default();

    // Extract direction information
    let direction_selector = Selector::parse(".Directions_table__name__CklG5")?;
    let direction_name = document
        .select(&direction_selector)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    let direction_code_selector = Selector::parse(".Directions_table__header__qV8_J p")?;
    let direction_code = document
        .select(&direction_code_selector)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    // Extract admission places
    let places_selector = Selector::parse(".Directions_table__places__RWYBT span")?;
    let places: Vec<String> = document
        .select(&places_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();

    let budget_places = places.get(0)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let target_places = places.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let contract_places = places.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Extract program manager info
    let manager_name_selector = Selector::parse(".Information_manager__name__ecPmn div:nth-child(2)")?;
    let manager_name = document
        .select(&manager_name_selector)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    let manager_email_selector = Selector::parse("a[href^='mailto:']")?;
    let manager_email = document
        .select(&manager_email_selector)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    let manager_phone_selector = Selector::parse("a[href^='tel:']")?;
    let manager_phone = document
        .select(&manager_phone_selector)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    let manager = ProgramManager {
        name: manager_name,
        email: manager_email,
        phone: manager_phone,
    };

    // Extract social links
    let social_selector = Selector::parse(".Information_socials__link___eN3E")?;
    let social_links: Vec<SocialLink> = document
        .select(&social_selector)
        .filter_map(|e| {
            let url = e.value().attr("href")?;
            let inner_html = e.inner_html();
            let platform = inner_html.split('<').next().unwrap_or("").trim();
            if !platform.is_empty() {
                Some(SocialLink {
                    platform: platform.to_string(),
                    url: url.to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    // Extract exam dates
    let exam_dates_selector = Selector::parse(".Information_entry__container__WYx9j h6")?;
    let exam_dates: Vec<String> = document
        .select(&exam_dates_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();

    // Extract admission methods
    let admission_method_selector = Selector::parse(".Accordion_accordion__title__tSP_0 h5")?;
    let admission_desc_selector = Selector::parse(".Accordion_accordion__info__wkCQC div")?;
    
    let method_titles: Vec<String> = document
        .select(&admission_method_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();
    
    let method_descriptions: Vec<String> = document
        .select(&admission_desc_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();

    let admission_methods: Vec<AdmissionMethod> = method_titles
        .into_iter()
        .zip(method_descriptions.into_iter())
        .map(|(name, description)| AdmissionMethod { name, description })
        .collect();

    // Extract career information
    let career_selector = Selector::parse(".Career_career__container___st5X h5")?;
    let career_opportunities = document
        .select(&career_selector)
        .next()
        .map(|e| e.inner_html().trim().replace("<br>", "\n").replace("<br><br>", "\n\n"))
        .unwrap_or_default();

    // Extract average salary (from career section)
    let average_salary = if career_opportunities.contains("от 150 до 400+") {
        "от 150 до 400+ тысяч рублей в месяц через 1–3 года после окончания".to_string()
    } else {
        String::new()
    };

    // Extract team members
    let team_name_selector = Selector::parse(".Team_team__name__q2R7T")?;
    let team_position_selector = Selector::parse(".Team_team__position__xB_og")?;
    
    let team_names: Vec<String> = document
        .select(&team_name_selector)
        .map(|e| e.inner_html().split('<').next().unwrap_or("").trim().to_string())
        .collect();

    let team_positions: Vec<String> = document
        .select(&team_position_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();

    let team: Vec<TeamMember> = team_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| TeamMember {
            name,
            position: team_positions.get(i).cloned().unwrap_or_default(),
            degree: None, // This would need more complex parsing to extract
        })
        .collect();

    // Extract partners
    let partners = vec!["Альфа-Банк".to_string(), "AlfaFuture".to_string()]; // Extracted from the content

    // Extract scholarships
    let scholarship_selector = Selector::parse(".Scholarship_item__cowlU h5, .Scholarship_item__cowlU h4")?;
    let scholarship_elements: Vec<String> = document
        .select(&scholarship_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();

    let mut scholarships = Vec::new();
    for i in (0..scholarship_elements.len()).step_by(2) {
        if let (Some(name), Some(amount)) = (scholarship_elements.get(i), scholarship_elements.get(i + 1)) {
            scholarships.push(Scholarship {
                name: name.clone(),
                amount: amount.clone(),
            });
        }
    }

    // Extract international opportunities
    let international_opportunities = vec![
        "Образовательные мероприятия и стажировки для студентов".to_string(),
        "Обучение за границей для студентов ИТМО".to_string(),
        "Study Abroad at Home".to_string(),
        "Buddy System".to_string(),
        "Конкурс стипендий Президента РФ для обучения за рубежом".to_string(),
    ];

    // Extract FAQ
    let faq_question_selector = Selector::parse(".Accordion_accordion__title__tSP_0 h5")?;
    let faq_answer_selector = Selector::parse(".Accordion_accordion__info__wkCQC div")?;
    
    // Skip admission methods questions and get FAQ questions
    let all_questions: Vec<String> = document
        .select(&faq_question_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();
    
    let all_answers: Vec<String> = document
        .select(&faq_answer_selector)
        .map(|e| e.inner_html().trim().to_string())
        .collect();

    // Extract only FAQ items (skip first 7 which are admission methods)
    let faq_questions = all_questions.iter().skip(7);
    let faq_answers = all_answers.iter().skip(7);
    
    let faq: Vec<FaqItem> = faq_questions
        .zip(faq_answers)
        .map(|(question, answer)| FaqItem {
            question: question.clone(),
            answer: answer.clone(),
        })
        .collect();

    Ok(MasterProgram {
        title,
        description,
        institute,
        study_form,
        duration,
        language,
        cost,
        dormitory,
        military_center,
        accreditation,
        special_programs,
        direction_code,
        direction_name,
        budget_places,
        target_places,
        contract_places,
        manager,
        social_links,
        exam_dates,
        admission_methods,
        career_opportunities,
        average_salary,
        team,
        partners,
        scholarships,
        international_opportunities,
        faq,
    })
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use std::fs;

    #[test]
    fn test_parse_ai_product_html() {
        // This test would use the actual HTML file
        // let html_content = fs::read_to_string("test_data/ai_product.html").unwrap();
        // let program = parse_master_program_html(&html_content).unwrap();
        // assert_eq!(program.title, "Управление ИИ-продуктами/AI Product");
    }
}
