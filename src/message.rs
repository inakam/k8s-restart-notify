use serde_json::json;

/// GKEコンソールURLのベース
const GKE_CONSOLE_BASE_URL: &str = "https://console.cloud.google.com/kubernetes/pod";

#[derive(Debug)]
pub struct ContainerRestartInfo {
    pub namespace: Option<String>,
    pub pod_name: String,
    pub container_name: String,
    pub container_image: String,
    pub node_name: Option<String>,
    pub restart_count: i32,
    pub last_state: Option<ContainerState>,
    pub resources: ContainerResources,
    pub logs: ContainerLog,
    pub channel: String,
    pub region: String,
    pub project_id: String,
}

impl ContainerRestartInfo {
    pub fn to_message(&self, file_url: &Option<String>) -> serde_json::Value {
        let gke_link = self.build_gke_link();
        
        // Slackに合わせて大まかに揃うようにする
        let container_identity = format!(
            r"Namespace:        {}
Pod:                   `{}`
Container Name: `{}`
Image:               `{}`
Node:                 {}
Reason:              {}",
            format_name(&self.namespace),
            &self.pod_name,
            &self.container_name,
            &self.container_image,
            format_name(&self.node_name),
            format_name(&self.last_state.as_ref().unwrap().reason),
        );
        
        let primary_fields = vec![
            markdown_text(&format!("Restart Count: `{}`", self.restart_count)),
        ];
        
        let mut blocks = vec![
            json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": "Container restarted",
                },
            }),
            json!({
                "type": "section",
                "text": markdown_text(&container_identity),
            }),
            json!({
                "type": "section",
                "fields": primary_fields,
            }),
            json!({
                "type": "actions",
                "elements": [
                    {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "Go to GKE Console",
                        },
                        "url": gke_link,
                        "style": "primary"
                    }
                ]
            }),
        ];
        
        // ログへのリンク
        if let Some(url) = file_url {
            blocks.push(json!({
                "type": "section",
                "text": markdown_text(&format!("<{}|*Check the logs before restart*>", url)),
            }));
        }
        
        json!(blocks)
    }
    
    /// GKEコンソールへのリンクを生成
    fn build_gke_link(&self) -> String {
        let namespace = self.namespace.as_deref().unwrap_or("default");
        // https://console.cloud.google.com/kubernetes/pod/<region>/<project_id>/<namespace>/<pod_name>/details を組み立てる
        format!("{}/{}/{}/{}/{}/details", GKE_CONSOLE_BASE_URL, self.region, self.project_id, namespace, self.pod_name)
    }
}

impl std::fmt::Display for ContainerRestartInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{} - {}",
            self.namespace.as_deref().unwrap_or(""),
            self.pod_name,
            self.container_name,
        )
    }
}

#[derive(Debug)]
pub struct ContainerState {
    pub exit_code: i32,
    pub signal: Option<i32>,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Default)]
pub struct ContainerResources {
    pub limits: Vec<(String, String)>,
    pub requests: Vec<(String, String)>,
}

impl ContainerResources {
    fn to_message(&self) -> Vec<serde_json::Value> {
        if self.limits.is_empty() && self.requests.is_empty() {
            return vec![markdown_text("No resource limits or requests")];
        }
        let mut message = Vec::new();
        for (resource, quantity) in &self.limits {
            message.push(markdown_text(&format!("{resource} limit: `{quantity}`")));
        }
        for (resource, quantity) in &self.requests {
            message.push(markdown_text(&format!("{resource} request: `{quantity}`")));
        }
        message
    }
}

#[derive(Debug)]
pub struct ContainerLog(pub Result<String, String>);

fn format_name(name: &Option<impl AsRef<str>>) -> String {
    if let Some(name) = name.as_ref() {
        format!("`{}`", name.as_ref())
    } else {
        "Unknown".to_owned()
    }
}

fn markdown_text(text: &str) -> serde_json::Value {
    json!({
        "type": "mrkdwn",
        "text": text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suffix() {
        assert_eq!(suffix("hello", 6), "hello");
        assert_eq!(suffix("hello", 5), "hello");
        assert_eq!(suffix("hello", 4), "ello");
        assert_eq!(suffix("hello", 3), "llo");
        assert_eq!(suffix("hello", 2), "lo");
        assert_eq!(suffix("hello", 1), "o");
        assert_eq!(suffix("hello", 0), "");
    }

    #[test]
    fn test_suffix_multibyte() {
        assert_eq!(suffix("こんにちは", 6), "こんにちは");
        assert_eq!(suffix("こんにちは", 5), "こんにちは");
        assert_eq!(suffix("こんにちは", 4), "んにちは");
        assert_eq!(suffix("こんにちは", 3), "にちは");
        assert_eq!(suffix("こんにちは", 2), "ちは");
        assert_eq!(suffix("こんにちは", 1), "は");
        assert_eq!(suffix("こんにちは", 0), "");
    }
    
    /// テスト用の関数
    fn suffix(text: &str, limit: usize) -> &str {
        if limit == 0 {
            return "";
        }
        // string slicing is in bytes, not chars, so we need to count chars
        let char_count = text.chars().count();
        if let Some(begin_char) = char_count.checked_sub(limit) {
            let begin_char = begin_char.min(char_count);
            let begin_byte = text.char_indices().nth(begin_char).unwrap().0;
            &text[begin_byte..]
        } else {
            text
        }
    }
}
