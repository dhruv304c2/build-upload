use reqwest::blocking::Client;
use serde_json::json;
use std::error::Error;

pub fn send_slack_message(token: &str, channel: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = "https://slack.com/api/chat.postMessage";
    let payload = json!({
        "channel": channel,
        "text": message,
    });
    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&payload)
        .send()?;
    if response.status().is_success() {
        let body: serde_json::Value = response.json()?;
        if body["ok"].as_bool().unwrap_or(false) {
            Ok(())
        } else {
            let error_message = body["error"].as_str().unwrap_or("Unknown error");
            Err(format!("failed to send message: {}", error_message).into())
        }
    } else {
        let status = response.status();
        Err(format!("Slack API request failed with status {}", status).into())
    }
}
