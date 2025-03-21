# k8s-restart-notify kustomize example

Kubernetes クラスタ内の Pod の再起動を監視し、Slack に通知するツールを kustomize でデプロイする例。

## namespace の作成

```bash
kubectl create namespace <YOUR_NAMESPACE>
```

## シークレットの作成

Slack API トークンは以下のようにシークレットとして作成する必要があります：

```bash
kubectl create secret generic k8s-restart-notify-slack-api-token \
  --from-literal=token=YOUR_SLACK_API_TOKEN \
  -n <YOUR_NAMESPACE>
```

## デプロイ

```bash
kubectl apply -k deployment/kustomize_example/dev
```
