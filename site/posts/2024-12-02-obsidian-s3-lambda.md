---
title: Obsidianの変更をトリガーにAWS Lambdaでプログラムを実行する
date: 2024-12-02 16:38
tags:
  - obsidian
  - aws
---
こんにちは。

みなさんはメモアプリに何を使っていますか？
私が使っている[Obsidian](https://obsidian.md/)はmarkdownですべてのメモを記述できる、アプリによってメモ間の関連付けが簡単に行える、サードパーティのプラグインが充実していてさまざまな機能を実現できるなどの魅力があり、1年以上使っています。

## Obsidianの魅力

ここからは知らない方のためにObsidianや関連プラグインの説明をしていきます。
私は知ってるよ！という方はこの節は読み飛ばしていただいて大丈夫です。

Obsidianではmarkdownが簡単に記述できるのが魅力です。
WYSIWYGスタイルのエディタが含まれており、書いたmarkdownはすぐさま視覚的にみやすい形になります。

Obsidianでは、公式がObsidian Syncというサービスを提供しており、有料契約を行うことで、端末間で同期を行なってくれます。
一方で、サードパーティのプラグインにもこうした同期を行うためのものがあります。その一つがremotely-saveです。
remotely-saveではさまざまなクラウドサービスにObsidianの内容を同期することができます。
そのため、自分の契約しているクラウドサービスなどと同期を行うこともできます。

私はOffice 365のビジネス版を契約しているので、最初はOneDrive（正確にはSharePoint）と同期をしていました。
しかし、しばらく使ってみて、remotely-saveは一番最初の候補にもなっているS3互換のAPIとの同期が強いことがわかりました。
私はS3もWasabiもCloudflare R2も使ったことがあり、どれを使うか悩みましたが、過去にS3以外サポートしていないソフトウェアとの相性トラブルに見舞われたこともあって、今回はS3を選択しました。

ObsidianにはGitで同期を行うサードパーティのプラグインもありましたが、これはモバイルでの体験がよくなかったです。
メモリの制約により、そもそもpullやpushができない、したら落ちるなどの問題があり、すぐにモバイルでは使えないことがわかりました。
それはそうとして、メモの差分管理とかはしてみたいなとは思いますよね。

そこで着目したのがLambdaでした。

## S3の更新をトリガーにLambdaを実行する

AWSのLambdaはいわゆるサーバレスのプログラム実行環境で、さまざまなトリガーを起因としてプログラムを実行させることができます。
今回私はObsidianのサードパーティプラグインであるremotely-saveを用いて、Obsidianの同期をS3を利用してとっていました。
つまり、Obsidianの更新がリアルタイムでS3に反映されるということで、S3に変更が加わるということは、Lambdaをトリガーすることができます。

すなわち、ここでObsidianの更新をトリガーに、Lambda上で実行できるプログラムならなんでも実行することが判明しました。

ちなみにこれは新規性あるらしいです。
<blockquote class="twitter-tweet"><p lang="ja" dir="ltr">うわー、obsidianの更新でlambdaトリガーできるのアツ。なんで思いつかなかったんだろう。新規性なので論文書いたほうがいいですよ。</p>&mdash; ノーン (@nkowne63) <a href="https://twitter.com/nkowne63/status/1853415804257333669?ref_src=twsrc%5Etfw">November 4, 2024</a></blockquote> <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

remotely-saveでのS3の設定の仕方やS3をトリガーにしたLambdaの発火方法については他に記事がたくさんあると思うので、ここでは私の活用法について紹介します。

### LambdaでObsidianの内容をGitHubと同期する

私はObsidianの更新内容をGitで管理したいと考えており、GitHubに差分がpushできたら一番嬉しいという状況でした。

#### Pythonのプログラムを書く

そこで、今回はLambdaでGitHubのリポジトリをcloneし、そのファイルのmd5ハッシュ値とS3のETagを比較、差分のあるファイルのみS3からダウンロードして、 `git add -A` して、コミットとプッシュを行うプログラムをPythonで記述しました。
Gitを操作するため、依存関係には[GitPython](https://github.com/gitpython-developers/GitPython)などがいます。

環境はRyeを使って用意しましたが、この辺はなんでもいいと思います。

プログラムは少し長いので、[こちらを参照](https://github.com/cordx56/s3git)してください。
#### LambdaでGitが使えるDockerイメージを作る

Pythonのプログラムが書けたら、次はLambdaへデプロイを考えます。
もちろん、Lambdaの実行環境は色々と制約があり、そのままではGitを完全には活用できないため、Gitコマンドを使えるようなDockerイメージを作成する必要がありました。

上のリンクからDockerfileも見ることができます。

Dockerfileでは、まずGitを `amazonlinux:2023` のイメージにインストールし、それからGit本体や必要なライブラリを取り出しています。
次のステップで、Lambdaで使える `public.ecr.aws/lambda/python:3.12` のイメージに対して、前のステップからGit本体やライブラリをコピーしてきます。

Pythonプログラムやその依存関係も、イメージの中に固めます。

#### 実行

ここまでできたら、作成したイメージをECRにpushし、Lambdaの関数を作ります。
S3へのトリガーや環境変数を設定したら完了です。

## まとめ

今回は、Obsidianの変更をトリガーにAWS Lambdaのプログラムを実行できることを紹介し、その一例としてGitHubへの差分の自動バックアップを紹介しました。
時間のあるタイミングでCloudFormationなども作ろうかと思っているので、もし興味がある方がいればお声がけください。