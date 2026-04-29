use serde::{Deserialize, Serialize};

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

pub async fn rewrite(text: &str, tone: &str) -> anyhow::Result<String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let model = std::env::var("ANTHROPIC_MODEL")
        .unwrap_or_else(|_| "claude-haiku-4-5-20251001".to_string());

    let system = "You are a precise writing assistant. When given text and a tone, rewrite it in that tone. \
    Rules: preserve all original meaning and facts exactly; do not add, remove, or invent information; \
    use no emojis, no em dashes, no filler phrases; \
    output only the rewritten text with no preamble or explanation."
        .to_string();

    let prompt = format!("Tone: {}\n\nText:\n{}", tone, text);

    let body = RequestBody {
        model,
        max_tokens: 1024,
        system,
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let client = reqwest::Client::new();
    let response = client
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
