use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const MAX_INPUT_CHARS: usize = 8_000;
const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client")
});

#[derive(Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct RequestBody {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: &'static str,
}

#[derive(Deserialize)]
struct ResponseContent {
    text: String,
}

#[derive(Deserialize)]
struct ResponseBody {
    content: Vec<ResponseContent>,
}

fn validate(text: &str) -> anyhow::Result<()> {
    if text.len() > MAX_INPUT_CHARS {
        anyhow::bail!(
            "selected text is too long ({} chars, max {})",
            text.len(),
            MAX_INPUT_CHARS
        );
    }
    Ok(())
}

fn build_request(text: &str, tone: &str) -> RequestBody {
    RequestBody {
        model: std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string()),
        max_tokens: 1024,
        system: crate::prompts::SYSTEM,
        messages: vec![Message {
            role: "user",
            content: crate::prompts::user(tone, text),
        }],
    }
}

async fn call_api(api_key: &str, body: &RequestBody) -> anyhow::Result<ResponseBody> {
    let response = CLIENT
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("API error {}: {}", status, body);
    }

    response.json::<ResponseBody>().await.map_err(Into::into)
}

fn parse_response(response: ResponseBody) -> anyhow::Result<String> {
    response
        .content
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty response from API"))
        .map(|c| c.text)
}

pub async fn rewrite(text: &str, tone: &str) -> anyhow::Result<String> {
    validate(text)?;
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let body = build_request(text, tone);
    let response = call_api(&api_key, &body).await?;
    parse_response(response)
}
