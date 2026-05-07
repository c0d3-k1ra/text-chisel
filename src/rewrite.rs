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
    system: String,
    messages: Vec<Message>,
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
        system: crate::prompts::system(tone),
        messages: vec![Message {
            role: "user",
            content: crate::prompts::user(text),
        }],
    }
}

async fn call_api(api_key: &str, body: &RequestBody) -> anyhow::Result<ResponseBody> {
    log::debug!(
        "calling API: model={} max_tokens={}",
        body.model,
        body.max_tokens
    );
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

pub async fn rewrite_with_key(text: &str, tone: &str, api_key: &str) -> anyhow::Result<String> {
    validate(text)?;
    let body = build_request(text, tone);
    let response = call_api(api_key, &body).await?;
    parse_response(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate ---

    #[test]
    fn validate_empty_ok() {
        assert!(validate("").is_ok());
    }

    #[test]
    fn validate_at_limit_ok() {
        assert!(validate(&"a".repeat(MAX_INPUT_CHARS)).is_ok());
    }

    #[test]
    fn validate_over_limit_err() {
        let err = validate(&"a".repeat(MAX_INPUT_CHARS + 1)).unwrap_err();
        assert!(err.to_string().contains("too long"));
    }

    // --- build_request ---

    #[test]
    fn build_request_shape() {
        let req = build_request("hello world", "Professional");
        assert_eq!(req.max_tokens, 1024);
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
    }

    #[test]
    fn build_request_content_contains_text() {
        let req = build_request("fix this sentence", "Concise");
        assert_eq!(req.messages[0].content, "fix this sentence");
        assert!(req.system.contains("Concise"));
        assert!(req.system.contains("Cut every word"));
    }

    // --- parse_response ---

    #[test]
    fn parse_response_returns_first_text() {
        let resp = ResponseBody {
            content: vec![ResponseContent {
                text: "rewritten".to_string(),
            }],
        };
        assert_eq!(parse_response(resp).unwrap(), "rewritten");
    }

    #[test]
    fn parse_response_empty_content_errs() {
        let resp = ResponseBody { content: vec![] };
        assert!(parse_response(resp).is_err());
    }

    // --- live API ---

    #[test]
    #[ignore = "Requires ANTHROPIC_API_KEY env var set. Makes a real API call."]
    fn test_rewrite_with_real_api() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(rewrite(
            "Please fix my grammer its not good.",
            "Professional",
        ));
        assert!(result.is_ok(), "API call failed: {:?}", result.err());
        let text = result.unwrap();
        assert!(!text.is_empty());
        println!("Rewritten: {}", text);
    }

    #[test]
    #[ignore = "Requires ANTHROPIC_API_KEY env var set. Verifies all five tones produce non-empty output."]
    fn test_all_tones_with_real_api() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let input = "hey can you send me the report before friday thanks";
        for tone in &["Professional", "Polite", "Assertive", "Concise", "Gen Z"] {
            let result = rt.block_on(rewrite(input, tone));
            assert!(result.is_ok(), "tone {} failed: {:?}", tone, result.err());
            assert!(!result.unwrap().is_empty(), "tone {} returned empty", tone);
            println!(
                "{}:\n{}\n",
                tone,
                rt.block_on(rewrite(input, tone)).unwrap()
            );
        }
    }
}
