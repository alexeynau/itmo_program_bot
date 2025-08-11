use serde::{Deserialize, Serialize};

// Структура для клиента Yandex SpeechKit
#[derive(Debug, Clone)]
pub struct YandexGPTClient {
    api_key: String,
    base_url: String,
    folder_id: String,
}

impl YandexGPTClient {
    pub fn new(api_key: String, base_url: String, folder_id: String) -> Self {
        YandexGPTClient {
            api_key,
            base_url,
            folder_id,
        }
    }

    pub async fn get_answer(&self, system_prompt: &str, user_text: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let modelUri = format!("gpt://{}/yandexgpt", self.folder_id);

        // Создаем JSON запрос
        let request_body = YandexGPTRequest {
            modelUri: modelUri,
            completionOptions: CompletionOptions {
                stream: false,
                temperature: 0.0,
                maxTokens: "4000".to_string(),
            },
            messages: vec![
                Message {
                    role: Role::system,
                    text: system_prompt.to_string(),
                },
                Message {
                    role: Role::user,
                    text: user_text.to_string(),
                },
            ],
        };

        let response = client
            .post(&self.base_url)
            .header("Authorization", format!("Api-Key {}", self.api_key))
            .json(&serde_json::json!(request_body))
            .send()
            .await?;

        if response.status().is_success() {
            let summary: YandexGPTResponse = response.json().await?;
            let text = summary.result.alternatives[0].message.text.clone();
            println!("request: {}", user_text);
            println!("system prompt: {}", system_prompt);
            println!("answer: {}", text);
            Ok(text)
        } else {
            Err(anyhow::anyhow!("Failed to get summary"))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub stream: bool,
    pub temperature: f32,
    pub maxTokens: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub text: String,
}
#[derive(Debug, Serialize, Deserialize)]
enum Role {
    system,
    user,
    assistant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YandexGPTRequest {
    pub modelUri: String,
    pub completionOptions: CompletionOptions,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YandexGPTResponse {
    pub result: ResultData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultData {
    pub alternatives: Vec<Alternative>,
    pub usage: Usage,
    pub modelVersion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alternative {
    pub message: Message,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub inputTextTokens: String,
    pub completionTokens: String,
    pub totalTokens: String,
}
