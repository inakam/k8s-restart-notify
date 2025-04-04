# k8s-restart-notify

[![Rust build and test](https://github.com/inakam/k8s-restart-notify/actions/workflows/rust.yml/badge.svg)](https://github.com/inakam/k8s-restart-notify/actions/workflows/rust.yml)

Fork of [flywheel-jp/johari-mirror](https://github.com/flywheel-jp/johari-mirror),
with some changes to make it work with my own Kubernetes cluster.

## Overview

k8s-restart-notify collects information about restarted containers and post notifications
to Slack like the following.

![Example Slack notification](docs/example-notification.png)

## Installation

You can use [example.yaml](deployment/example.yaml) to deploy k8s-restart-notify to your
Kubernetes cluster with `NAMESPACE` and `NOTIFICATION_CHANNEL` replaced.

```sh
kubectl create secret generic k8s-restart-notify-slack-api-token \
  --from-literal=token=<your-slack-token>
kubectl apply -f example.yaml
```

### Environment variables

All environment variables are required except `IGNORED_NAMESPACES`.

| Name                        | Description                                                               |
| :-------------------------- | :------------------------------------------------------------------------ |
| `SLACK_TOKEN`               | Slack Bot User OAuth Token. See Slack authentication section.             |
| `SLACK_NOTIFICATION_CONFIG` | Filters to configure notification destination. See the following section. |
| `REGION`                    | The region of the Kubernetes cluster.                                     |
| `PROJECT_ID`                | The project ID of the Kubernetes cluster.                                 |
| `CLUSTER_ID`                | The cluster ID of the Kubernetes cluster.                                 |
| `IGNORED_NAMESPACES`        | Optional. Comma-separated list of namespaces to ignore.                   |

#### SLACK_NOTIFICATION_CONFIG

`SLACK_NOTIFICATION_CONFIG` environment variable defines a list of rules to configure
notification destination delimited by commas in
`namespace/pod/container=channel,...,namespace/pod/container=channel` format.

- When a container restart is detected, k8s-restart-notify determines the Slack channel
  to send notification by its `namespace`, `pod` name and `container` name.
- Earlier rules have higher priority.
- Each of `namespace`, `pod` or `container` in a rule may contain `*` wildcards.
- `channel` can be either of a Slack channel name, a Slack channel ID or
  an empty string. Empty string suppresses notification.

Examples

- `*/*/*=monitoring`
  - Any container restarts are notified to `monitoring` Slack channel.
- `kube-system/coredns-*/*=monitoring-coredns,kube-system/*/*=,*/*/*=monitoring`
  - Restarts of pods beginning with `coredns-` in `kube-system` namespace are notified
    to `monitoring-coredns` channel.
  - Restarts of other pods in `kube-system` namespace are not notified.
  - Restarts in the other namespaces are notified to `monitoring` channel.

#### IGNORED_NAMESPACES

`IGNORED_NAMESPACES` environment variable defines a comma-separated list of namespaces
to ignore. Container restarts in these namespaces will not trigger notifications.

Examples:

- `kube-system,monitoring,logging`
  - Restarts in `kube-system`, `monitoring`, and `logging` namespaces will be ignored.
- `kube-system, monitoring, logging`
  - Spaces around commas are allowed and will be trimmed.

This configuration is useful for reducing noise from system namespaces that have frequent
restarts which aren't relevant to application monitoring.

### Slack authentication

Ref: [Quickstart | Slack](https://api.slack.com/start/quickstart)

Create a Slack App and install it to your workspace.
k8s-restart-notify uses
[`Bot User OAuth Token`](https://api.slack.com/authentication/token-types#bot)
in the environment variable `SLACK_TOKEN`.

#### Required permission scopes

- Bot Token Scopes
  - `chat:write.public` or `chat:write`
    - With `chat:write`, the app needs to be invited to the target Slack channels.
  - `files:write`

### Kubernetes authentication

Kubernetes authentication can be obtained from `KUBECONFIG`, `~/.kube/config` or
in-cluster config.

Ref. [Config in kube - Rust](https://docs.rs/kube/latest/kube/struct.Config.html#method.infer)

See [example manifest](deployment/example.yaml) for authentication using ServiceAccount.

#### Required permissions

- Resources: `pods`, `pods/log`
- Verbs: `get`, `watch`, `list`

## License

MIT

## Related projects

- [airwallex/k8s-pod-restart-info-collector](https://github.com/airwallex/k8s-pod-restart-info-collector)
- [flywheel-jp/johari-mirror](https://github.com/flywheel-jp/johari-mirror)
