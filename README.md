# under-maintenance / under-maintenance

## Table of Contents / 目次

- [English](#english)
  - [Overview](#overview)
  - [Development](#development)
  - [Container Image](#container-image)
  - [Helm Chart](#helm-chart)
  - [Release](#release)
- [日本語](#日本語)
  - [概要](#概要)
  - [開発](#開発)
  - [コンテナイメージ](#コンテナイメージ)
  - [Helm チャート](#helm-チャート)
  - [リリース](#リリース)

## English

### Overview

`under-maintenance` is a small static SPA for maintenance mode. It shows a localized SVG hero and a single language switch button.

### Development

```bash
npm install
npm run build
npm run preview
```

### Container Image

The Docker image uses a multi-stage build: Vite builds the Svelte app, then the Rust `client-minihttp` server serves the generated static files.

```bash
docker build -t under-maintenance:local .
docker run --rm -p 8080:8080 under-maintenance:local
```

### Helm Chart

The chart is in `deploy/chart`.

```bash
helm template under-maintenance deploy/chart
helm template under-maintenance deploy/chart --set traefik.enabled=true
```

The chart supports standard Kubernetes Ingress and Traefik `IngressRoute`.

### Release

GitHub Actions release automation is generated under `.github/workflows/release.yml`. Releases are computed from merged `feature/*` pull request commit messages.

## 日本語

### 概要

`under-maintenance` は、メンテナンスモードを表示する小さな静的 SPA です。言語別の SVG hero と、言語切り替えボタンだけを表示します。

### 開発

```bash
npm install
npm run build
npm run preview
```

### コンテナイメージ

Docker image は multi-stage build です。Vite で Svelte app をビルドし、生成された静的ファイルを Rust の `client-minihttp` server で配信します。

```bash
docker build -t under-maintenance:local .
docker run --rm -p 8080:8080 under-maintenance:local
```

### Helm チャート

chart は `deploy/chart` にあります。

```bash
helm template under-maintenance deploy/chart
helm template under-maintenance deploy/chart --set traefik.enabled=true
```

chart は Kubernetes の通常 Ingress と Traefik `IngressRoute` に対応しています。

### リリース

GitHub Actions の release automation は `.github/workflows/release.yml` に生成されます。Release は merge 済み `feature/*` pull request の commit message から計算されます。
