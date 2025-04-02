use kube::Client;
use tokio::sync::mpsc;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("k8s-restart-notify=debug"),
    )
    .init();

    // Infer the runtime environment and try to create a Kubernetes Client
    let client = Client::try_default().await?;

    let slack_token = std::env::var("SLACK_TOKEN")?;
    let region = std::env::var("REGION")?;
    let project_id = std::env::var("PROJECT_ID")?;
    let cluster_id = std::env::var("CLUSTER_ID")?;
    
    // 無視するNamespaceのリストを取得
    let ignored_namespaces = std::env::var("IGNORED_NAMESPACES").ok()
        .map(|ns| {
            ns.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<HashSet<String>>()
        })
        .unwrap_or_default();
    
    if !ignored_namespaces.is_empty() {
        log::info!("Ignoring namespaces: {:?}", ignored_namespaces);
    }

    let (tx, rx) = mpsc::channel(320);
    let watch_handle = tokio::spawn(k8s_restart_notify::kubernetes::watch(
        client, 
        tx, 
        region, 
        project_id, 
        cluster_id, 
        ignored_namespaces
    ));
    let slack_handle = tokio::spawn(k8s_restart_notify::slack::slack_send(slack_token, rx));

    watch_handle.await??;
    slack_handle.await?;

    Ok(())
}
