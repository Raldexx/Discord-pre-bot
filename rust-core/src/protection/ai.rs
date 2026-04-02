use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ToxicityRequest {
    text: String,
}

#[derive(Deserialize)]
pub struct ToxicityResult {
    pub is_toxic: bool,
    pub score: f32,
    pub reason: String,
}

pub async fn check_toxicity(
    client: &reqwest::Client,
    ai_url: &str,
    text: &str,
) -> Result<ToxicityResult, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/analyze", ai_url);
    let response: reqwest::Response = client
        .post(&url)
        .json(&ToxicityRequest { text: text.to_string() })
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Ok(ToxicityResult {
            is_toxic: false,
            score: 0.0,
            reason: String::new(),
        })
    }
}
