use teloxide::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
struct ProgramData {
    ai: Program,
    ai_product: Program,
}
#[derive(Deserialize, Clone)]
struct Program {
    url: String,
    courses: Vec<String>,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    let data: ProgramData = serde_json::from_str(&fs::read_to_string("data/programs.json").unwrap()).unwrap();

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let data = data.clone();
        async move {
            let text = msg.text().unwrap_or_default().to_string();
            log::info!("Received message: {}", text);
            let answer = get_answer_from_llm(&text, &data).await.unwrap_or_else(|| {
                "Могу отвечать только по магистратурам AI и AI Product.".to_string()
            });

            bot.send_message(msg.chat.id, answer).await?;
            Ok(())
        }
    })
    .await;
}

async fn get_answer_from_llm(user_text: &str, data: &ProgramData) -> Option<String> {
    let system_prompt = format!(
        "Ты консультант по магистратурам ITMO. Вот данные: AI: {:?}, AI Product: {:?}. Отвечай строго на их основе. Если вопрос не по теме, скажи, что не можешь ответить.",
        data.ai.courses, data.ai_product.courses
    );

    // Здесь можно вставить запрос к OpenAI API или другому LLM
    // Пока для MVP — просто эхо
    if user_text.to_lowercase().contains("ai") {
        Some(format!("Похоже, вам интересна AI программа. Вот курсы: {:?}", data.ai.courses))
    } else if user_text.to_lowercase().contains("product") {
        Some(format!("Похоже, вам интересна AI Product программа. Вот курсы: {:?}", data.ai_product.courses))
    } else {
        None
    }
}
