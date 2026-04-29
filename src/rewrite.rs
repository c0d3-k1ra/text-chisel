use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client")
});

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct RequestBody {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: String,
}

#[derive(Deserialize)]
struct ResponseContent {
    text: String,
}

#[derive(Deserialize)]
struct ResponseBody {
    content: Vec<ResponseContent>,
}

const MAX_INPUT_CHARS: usize = 8_000;

pub async fn rewrite(text: &str, tone: &str) -> anyhow::Result<String> {
    if text.len() > MAX_INPUT_CHARS {
        anyhow::bail!(
            "selected text is too long ({} chars, max {})",
            text.len(),
            MAX_INPUT_CHARS
        );
    }
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let model = std::env::var("ANTHROPIC_MODEL")
        .unwrap_or_else(|_| "claude-haiku-4-5-20251001".to_string());

    let system = crate::prompts::SYSTEM.to_string();
    let prompt = crate::prompts::user(tone, text);

    let body = RequestBody {
        model,
        max_tokens: 1024,
        system,
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let response = CLIENT
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<ResponseBody>()
        .await?;
    let result = response
        .content
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty response from API"))?
        .text;

    Ok(result)
}
