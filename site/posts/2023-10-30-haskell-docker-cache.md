---
title: HaskellプロジェクトのDockerイメージビルドをいい感じにする
date: 2023-10-30 08:47
tags: [haskell, docker]
---

こんにちは。
技術書典が近いのに進捗がやばいです。

今回はHaskellプロジェクトをDockerでビルドする際に、よりキャッシュを効かせる方法などについてお話しします。
また、HaskellプロジェクトのDockerイメージをGitHub Actionsでビルドする方法についても少し話します。

Haskell自体情報が少なめですが、Dockerの情報はあまりなかったので、簡単に例を見せながらキャッシュを活かしたDockerイメージビルドについて解説します。

## Dockerのキャッシュ
Dockerはイメージのビルド時に、途中までの結果をキャッシュとして保存します。
そして変更がない部分に関しては過去にビルドされた時に作成したキャッシュを利用することでビルド時間を短縮することができます。

Haskellのビルドは時間がかかるのでキャッシュを利用してビルド時間を短縮したいですが、Dockerでビルド時間を短縮するようなDockerfileを書くのには少し考えることがあります。

ここでは、HaskellでstackとHPackを利用したプロジェクトでキャッシュを最大限利用するためのDockerイメージについて説明します。

いきなりですがDockerfileの例を提示します。
例を見ながら説明していきましょう。
なお、このファイルは[このブログのビルドにも利用しているプロジェクトのDockerfile](https://github.com/cordx56/slick-site-builder/blob/main/Dockerfile)とほぼ同じです。

```
FROM haskell:9.4.7 AS builder
WORKDIR /app
COPY package.yaml .
COPY stack.yaml .
COPY stack.yaml.lock .
RUN stack install --only-dependencies
COPY . .
RUN stack install --local-bin-path /

FROM debian:buster-slim
WORKDIR /app
ENV LC_ALL C.UTF-8
COPY --from=builder /project-name /
CMD ["/project-name"]
```

こんな感じです。
一行ずつ見ていきましょう。
まず一行目はHaskellプロジェクトをビルドするためのイメージを指定します。

次にワーキングディレクトリを指定します。
これは別に必要ないですが、適当にやっておきます。

その次に `package,yaml` 、 `stack.yaml` 、 `stack.yaml.lock` をビルドイメージにコピーします。
この3つのみを先にコピーしておくことにより、依存関係が変わっていなければ過去のビルドキャッシュを利用することができます。

次に `stack install --only-dependencies` の実行です。
これにより、依存しているもののみビルドすることができます。

ここまでが再利用したいキャッシュになります。
この部分のキャッシュを使い回すことにより、ビルド時間を短縮します。

`stack install --local-bin-path /` を実行することにより、プロジェクトをビルドし、結果得られる実行ファイルをルートディレクトリに配置します。

ここまでがビルドです。

ビルドが完了したら、Dockerイメージサイズを小さくするために、マルチステージビルドを活用していきましょう。
適当なDebianのDockerイメージに、先ほどビルドした実行ファイルをコピーして、 `CMD` 命令でそのファイルを指定すれば完了です。

## HaskellプロジェクトのDockerイメージをビルドするGitHub Actions Workflow

ここまでで、Dockerイメージのビルドについて説明してきました。
続いて、[ブログのビルダのGitHub Actions Workflow](https://github.com/cordx56/slick-site-builder/blob/main/.github/workflows/build.yaml)を眺めながら、Haskellを利用したDockerイメージのビルドをGitHub Actionsで行う方法についても言及します。

実際のworkflowがこちらです。

```
name: Build and publish

on:
  push:
    branches:
      - main

env:
  REGISTRY: ghcr.io

jobs:
  build:
    name: Build and publish Docker image
    runs-on: ubuntu-latest
    permissions:
      packages: write
    steps:
      - uses: actions/checkout@v4

      - id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ github.repository }}
      
      - name: Setup buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

重要なポイントは[docker/build-push-action](https://github.com/docker/build-push-action)に、 `cache-from: type=gha` と `cache-to: type=gha,mode=max` を指定することです。
これにより、GitHub ActionsでDockerビルドのキャッシュを保存できます。

## まとめ
以上がHaskellプロジェクトでDockerイメージをビルドする際に、キャッシュを効かせていくやり方の例です。
この取り組みで、私のプロジェクトではGitHub ActionsでDockerイメージビルド全体に30分かかるプロジェクトを2分でビルドすることができるようになりました。
実際のプロジェクトではもう少し複雑なことをする必要があるかもしれませんが、このベースを元にやっていけば、キャッシュを活用できるはずです。

以上、HaskellとDockerイメージビルドのキャッシュのお話でした。
