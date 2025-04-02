use anyhow::bail;
use chrono::Utc;
use serde_json::json;
use tokio::sync::mpsc;

use crate::message;

const POST_MESSAGE_URL: &str = "https://slack.com/api/chat.postMessage";

/// Task to send messages to Slack channel
pub async fn slack_send(
    slack_token: String,
    mut rx: mpsc::Receiver<message::ContainerRestartInfo>,
) {
    let slack = reqwest::Client::new();

    while let Some(restart_info) = rx.recv().await {
        log::debug!("Start sending message to Slack: {restart_info}");
        if let Err(e) = post_notification(&slack, &slack_token, &restart_info).await {
            log::error!("Failed to post message to Slack: {e}");
        }
        log::debug!("Finished sending message to Slack: {restart_info}");
    }
}

async fn post_notification(
    slack: &reqwest::Client,
    slack_token: &str,
    restart_info: &message::ContainerRestartInfo,
) -> anyhow::Result<()> {
    post_message(slack, slack_token, &restart_info.channel, restart_info).await
}

async fn post_message(
    slack: &reqwest::Client,
    slack_token: &str,
    slack_channel: &str,
    restart_info: &message::ContainerRestartInfo,
) -> anyhow::Result<()> {
    let message = serde_json::json!({
        "channel": slack_channel,
        "blocks": restart_info.to_message(),
        "unfurl_links": false,
        "unfurl_media": false,
    });
    let resp = slack
        .post(POST_MESSAGE_URL)
        .bearer_auth(slack_token)
        .json(&message)
        .send()
        .await?;
    parse_slack_response(resp).await?;
    Ok(())
}

async fn parse_slack_response(resp: reqwest::Response) -> anyhow::Result<serde_json::Value> {
    if !resp.status().is_success() {
        bail!(
            "Slack API failed: {}",
            resp.text().await.unwrap_or_else(|err| err.to_string())
        );
    }
    log::debug!("Response from Slack: status={}", resp.status());
    let resp: serde_json::Value = resp.json().await?;
    if !matches!(resp.get("ok"), Some(serde_json::Value::Bool(true))) {
        if let Some(error) = resp.get("error") {
            bail!("Slack response is not ok: {}", error);
        } else {
            bail!("Unexpected Slack response format: {:?}", resp);
        }
    }
    Ok(resp)
}
