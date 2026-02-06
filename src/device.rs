use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

const CANVAS_SIZE: u32 = 64;

fn url(ip: &str) -> String {
    format!("http://{}/post", ip)
}

async fn post_command(ip: &str, payload: Value) -> Result<Value> {
    let client = reqwest::Client::new();
    let resp = client
        .post(&url(ip))
        .json(&payload)
        .send()
        .await
        .context("Failed to send request to device")?;

    let body: Value = resp
        .json()
        .await
        .context("Failed to parse device response")?;

    Ok(body)
}

pub async fn push_image(ip: &str, rgb_data: &[u8]) -> Result<()> {
    let encoded = STANDARD.encode(rgb_data);
    let pic_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 10000;

    let payload = json!({
        "Command": "Draw/SendHttpGif",
        "PicNum": 1,
        "PicWidth": CANVAS_SIZE,
        "PicOffset": 0,
        "PicID": pic_id,
        "PicSpeed": 1000,
        "PicData": encoded,
    });

    post_command(ip, payload).await?;
    Ok(())
}

pub async fn get_settings(ip: &str) -> Result<()> {
    let payload = json!({ "Command": "Channel/GetIndex" });
    let channel = post_command(ip, payload).await?;
    println!("Channel: {}", serde_json::to_string_pretty(&channel)?);

    let payload = json!({ "Command": "Channel/GetAllConf" });
    let conf = post_command(ip, payload).await?;
    println!("Config: {}", serde_json::to_string_pretty(&conf)?);

    Ok(())
}
