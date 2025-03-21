use kube::Client;
use tokio::sync::mpsc;

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

    let (tx, rx) = mpsc::channel(320);
    let watch_handle = tokio::spawn(k8s_restart_notify::kubernetes::watch(client, tx, region, project_id));
    let slack_handle = tokio::spawn(k8s_restart_notify::slack::slack_send(slack_token, rx));

    watch_handle.await??;
    slack_handle.await?;

    Ok(())
}
