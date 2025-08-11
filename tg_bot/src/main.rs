use serde::{Deserialize, Serialize};
use std::fs;
use teloxide::prelude::*;

pub mod html_parser;
pub mod yandex_gpt_client;

#[derive(Serialize, Deserialize, Clone)]
struct ProgramData {
    ai: Program,
    ai_product: Program,
}
#[derive(Serialize, Deserialize, Clone)]
struct Program {
    url: String,
    courses: Vec<String>,
    info: Option<String>,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    dotenv::dotenv().ok();
    let api_key = dotenv::var("YANDEX_GPT_API_KEY").expect("YANDEX_GPT_API_KEY not set");
    let base_url = dotenv::var("YANDEX_GPT_API_URL").expect("YANDEX_GPT_API_URL not set");
    let folder_id = dotenv::var("YANDEX_FOLDER_ID").expect("YANDEX_FOLDER_ID not set");
    let teloxide_token = dotenv::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN not set");
    let bot = Bot::new(teloxide_token);
    let yandex_gpt_client = yandex_gpt_client::YandexGPTClient::new(api_key, base_url, folder_id);
    let mut data: ProgramData =
        serde_json::from_str(&fs::read_to_string("data/programs.json").unwrap()).unwrap();
    let ai_info = &fs::read_to_string("data/ai_parsed.json").unwrap();
    let ai_product_info = &fs::read_to_string("data/ai_product_parsed.json").unwrap();

    data.ai.info = Some(ai_info.clone());
    data.ai_product.info = Some(ai_product_info.clone());
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let data = data.clone();
        let yandex_client = yandex_gpt_client.clone();
        async move {
            let text = msg.text().unwrap_or_default().to_string();
            log::info!("Received message: {}", text);
            let answer = get_answer_from_llm(&text, &data, &yandex_client)
                .await
                .unwrap_or_else(|| {
                    "Могу отвечать только по магистратурам AI и AI Product.".to_string()
                });

            bot.send_message(msg.chat.id, answer).await?;
            Ok(())
        }
    })
    .await;
}

// Helper function to create a concise program summary
fn create_program_summary(program: &Program, program_name: &str) -> String {
    let info_summary = if let Some(info_str) = &program.info {
        match serde_json::from_str::<serde_json::Value>(info_str) {
            Ok(info_json) => {
                format!(
                    "Описание: {}. Стоимость: {}. Места: {} бюджетных. Форма: {}.",
                    info_json
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Нет описания"),
                    info_json
                        .get("cost")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Не указана"),
                    info_json
                        .get("budget_places")
                        .and_then(|v| v.as_u64())
                        .map(|n| n.to_string())
                        .unwrap_or("Не указано".to_string()),
                    info_json
                        .get("study_form")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Не указана")
                )
            }
            Err(_) => "Информация недоступна".to_string(),
        }
    } else {
        "Информация недоступна".to_string()
    };

    let courses_count = program.courses.len();
    format!(
        "{}: {}. Количество курсов: {}. URL: {}",
        program_name, info_summary, courses_count, program.url
    )
}

// Helper function to get relevant courses based on user query
fn get_relevant_courses(program: &Program, user_text: &str, max_courses: usize) -> Vec<String> {
    let user_lower = user_text.to_lowercase();
    let keywords = [
        "машинное обучение",
        "ml",
        "deep learning",
        "глубокое обучение",
        "python",
        "данные",
        "data",
        "нейронные сети",
        "neural",
        "ai",
        "ии",
        "статистика",
        "математика",
        "алгоритм",
        "веб",
        "web",
        "база данных",
        "database",
        "визуализация",
        "nlp",
        "обработка текста",
        "изображения",
        "gpu",
    ];

    let mut scored_courses: Vec<(String, i32)> = program
        .courses
        .iter()
        .map(|course| {
            let course_lower = course.to_lowercase();
            let score = keywords.iter().fold(0, |acc, keyword| {
                if user_lower.contains(keyword) && course_lower.contains(keyword) {
                    acc + 2
                } else if course_lower.contains(&user_lower) || user_lower.contains(&course_lower) {
                    acc + 1
                } else {
                    acc
                }
            });
            (course.clone(), score)
        })
        .collect();

    scored_courses.sort_by(|a, b| b.1.cmp(&a.1));
    scored_courses
        .into_iter()
        .take(max_courses)
        .map(|(course, _)| course)
        .collect()
}

/// Отправить вопрос пользователя и структуру ProgramData в LLM
/// и получить список полей, релевантных вопросу пользователя
async fn get_relevant_info(
    program: &Program,
    user_text: &str,
    yandex_client: &yandex_gpt_client::YandexGPTClient,
) -> anyhow::Result<String> {
    let fields: Vec<String> = vec![
        "title",
        "description",
        "institute",
        "study_form",
        "duration",
        "language",
        "cost",
        "dormitory",
        "military_center",
        "accreditation",
        "special_programs",
        "direction_code",
        "direction_name",
        "budget_places",
        "target_places",
        "contract_places",
        "manager",
        "social_links",
        "exam_dates",
        "admission_methods",
        "career_opportunities",
        "average_salary",
        "team",
        "partners",
        "scholarships",
        "international_opportunities",
        "faq",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let prompt: String = format!(
        "Проанализируй вопрос пользователя о магистерской программе и верни список полей, которые могут быть релевантны для ответа на вопрос: '{}'. Доступные поля: {}",
        user_text,
        fields.join(", ")
    );

    let response = yandex_client.get_answer("Ты LLM, который анализирует вопросы пользователей о магистерских программах и возвращает релевантные поля в виде JSON массива строк. ВАЖНО НЕ ИСПОЛЬЗУЙ форматирование markdown и ```", &prompt).await.inspect_err(|e| {
        println!("Error getting relevant fields: {}", e)
    })?;
    let relevant_fields: Vec<String> = serde_json::from_str(&response)
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    let program_info = program.info.clone().unwrap_or_default();
    let relevant_info =
        if let Ok(info_json) = serde_json::from_str::<serde_json::Value>(&program_info) {
            relevant_fields
                .iter()
                .filter_map(|field| {
                    info_json
                        .get(field)
                        .map(|value| format!("{}: {}", field, value))
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "Информация недоступна".to_string()
        };

    Ok(relevant_info)
}

async fn get_answer_from_llm(
    user_text: &str,
    data: &ProgramData,
    yandex_client: &yandex_gpt_client::YandexGPTClient,
) -> Option<String> {
    // Determine which program the user is asking about
    let user_lower = user_text.to_lowercase();
    let asking_about_ai_product = [
        "ai product",
        "ai-продукт",
        "ai product",
        "продукт",
        "Управление",
    ]
    .iter()
    .any(|&s| user_lower.contains(s));
    let asking_about_ai = [
        "ai",
        "искусственный интеллект",
        "машинное обучение",
        "глубокое обучение",
        "нейронные сети",
    ]
    .iter()
    .any(|&s| user_lower.contains(s));

    // Helper to build system prompt for a program
    async fn build_program_prompt(
        program: &Program,
        program_name: &str,
        user_text: &str,
        yandex_client: &yandex_gpt_client::YandexGPTClient,
    ) -> String {
        let summary = create_program_summary(program, program_name);
        let relevant_info = get_relevant_info(program, user_text, yandex_client)
            .await
            .unwrap_or_default();
        let relevant_courses = get_relevant_courses(program, user_text, 10).join(", ");
        format!(
            "Ты консультант по магистратуре {program_name} в ITMO. Программа:\n{summary}\n{relevant_courses}\nИнформация релевантная вопросу:\n{relevant_info}\nОтвечай кратко и по существу. Если вопрос не по теме, скажи что не можешь ответить."
        )
    }

    let system_prompt = if asking_about_ai_product == asking_about_ai {
        // User asking about both programs - provide summaries
        let ai_summary = create_program_summary(&data.ai, "AI");
        let ai_product_summary = create_program_summary(&data.ai_product, "AI Product");
        let ai_relevant_info = get_relevant_info(&data.ai, user_text, &yandex_client)
            .await
            .unwrap_or_default();
        format!(
            "Ты консультант по магистратурам ITMO. У нас есть 2 программы:\n{ai_summary}\n{ai_product_summary}\nИнформация релевантная вопросу:\n{ai_relevant_info}\nОтвечай кратко и по существу. Если вопрос не по теме, скажи что не можешь ответить."
        )
    } else if asking_about_ai_product {
        build_program_prompt(&data.ai_product, "AI Product", user_text, &yandex_client).await
    } else if asking_about_ai {
        build_program_prompt(&data.ai, "AI", user_text, &yandex_client).await
    } else {
        // General query - provide brief info about both
        "Ты консультант по магистратурам ITMO. У нас есть 2 AI программы: 'Искусственный интеллект' и 'AI Product'. Отвечай кратко. Если вопрос не по теме, скажи что не можешь ответить.".to_string()
    };

    // Используем Yandex GPT для получения ответа
    match yandex_client.get_answer(&system_prompt, user_text).await {
        Ok(response) => Some(response),
        Err(err) => {
            log::error!("Error getting answer from Yandex GPT: {}", err);
            // Fallback to simple logic if API fails
            if asking_about_ai_product {
                let courses = get_relevant_courses(&data.ai_product, user_text, 3);
                Some(format!(
                    "AI Product программа. Релевантные курсы: {}",
                    if courses.is_empty() {
                        "программирование, ML, продуктовая разработка".to_string()
                    } else {
                        courses.join(", ")
                    }
                ))
            } else if asking_about_ai {
                let courses = get_relevant_courses(&data.ai, user_text, 3);
                Some(format!(
                    "AI программа. Релевантные курсы: {}",
                    if courses.is_empty() {
                        "машинное обучение, глубокое обучение, Python".to_string()
                    } else {
                        courses.join(", ")
                    }
                ))
            } else {
                None
            }
        }
    }
}
