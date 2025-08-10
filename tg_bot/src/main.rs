use teloxide::prelude::*;
use serde::{ser, Deserialize, Serialize};
use std::fs;

pub mod yandex_gpt_client;
pub mod html_parser;

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
    let mut data: ProgramData = serde_json::from_str(&fs::read_to_string("data/programs.json").unwrap()).unwrap();
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
            let answer = get_answer_from_llm(&text, &data, &yandex_client).await.unwrap_or_else(|| {
                "Могу отвечать только по магистратурам AI и AI Product.".to_string()
            });

            bot.send_message(msg.chat.id, answer).await?;
            Ok(())
        }
    })
    .await;
}

async fn get_answer_from_llm(user_text: &str, data: &ProgramData, yandex_client: &yandex_gpt_client::YandexGPTClient) -> Option<String> {
    let ai: &String = &serde_json::to_string(&data.ai).unwrap();
    let ai_product: &String = &serde_json::to_string(&data.ai_product).unwrap();
    let system_prompt = format!(
        "Ты консультант по магистратурам ITMO. Вот данные: AI: {:?}, AI Product: {:?}. Отвечай строго на их основе. Если вопрос не по теме, скажи, что не можешь ответить.",
        ai, ai_product
    );

    // Используем Yandex GPT для получения ответа
    match yandex_client.get_answer(&system_prompt, user_text).await {
        Ok(response) => Some(response),
        Err(err) => {
            log::error!("Error getting answer from Yandex GPT: {}", err);
            // Fallback to simple logic if API fails
            if user_text.to_lowercase().contains("ai") && user_text.to_lowercase().contains("product") {
                Some(format!("Похоже, вам интересна AI Product программа. Вот курсы: {:?}", data.ai_product.courses))
            } else if user_text.to_lowercase().contains("ai") {
                Some(format!("Похоже, вам интересна AI программа. Вот курсы: {:?}", data.ai.courses))
            } else {
                None
            }
        }
    }
}
