---
title: HaskellでOGP画像も自動生成した
date: 2023-11-25 02:02
tags: [haskell]
---

こんにちは。
学位論文の締切の季節が近づいてきましたね。
私も修論の進捗がやばいです。

さて、そんな中ではありますが、今回はHaskellでOGP画像を自動生成してHakyllのサイト生成に乗っけたので、その話を書こうと思います。
モチベーションとしては、静的サイトでもOGP画像が欲しいよね、というだけです。

## 画像生成
OGP画像の生成ということであれば当然文字の描画は避けて通れないのですが、Haskellでフォントのレンダリングについて調べると、あまり状況が良くないことはすぐにわかると思います。
今回は [SVGFonts](https://hackage.haskell.org/package/SVGFonts) というライブラリを利用して文字を描画することにしました。
SVGFontsは [diagrams](https://hackage.haskell.org/package/diagrams) というライブラリの上で動くようなので、こちらについても確認する必要があります。

実際に文字や図形の描画を行うコードは [app/Image.hs](https://github.com/cordx56/blog.cordx.cx/blob/main/app/Image.hs) にあります。
基本的にはSVGFontsのドキュメント通りです。

SVGフォントはNoto Sansを元にFontForgeで作成しました。
GitHubにライセンスなども同梱しています。

```bash
fontforge -lang ff -c 'Open($1); Generate($2)' ~/Downloads/Noto_Sans_JP/static/NotoSansJP-Light.ttf ./noto.svg
```

のようにすれば生成できるはずです。
この方法で生成したSVGファイルですが、私の手元では一部のglyph要素のunicode属性が壊れていたので、その前にあるglyph-name属性の値を利用して置換を行いました。
置換はNeoVimで正規表現を使ってやりましたが、簡単にできるのでもし必要であればお手元の好きな環境でやれば良いと思います。

## Hakyllと組み合わせる
ここまでで画像の生成ができるようになったので、ここからはHakyllとの組み合わせの話をします。
実際のコードは [app/Main.hs](https://github.com/cordx56/blog.cordx.cx/blob/main/app/Main.hs) にあります。

重要なポイントは

- Hakyllの `unsafeCompiler` を使ってOGP画像を生成するComilerを作る
    - `TmpFile path <- newTmpFile` のようにして一時ファイルを作成し、そのパスに画像を生成する
    - 最後に `makeItem` と `CopyFile` で画像ファイルをターゲットとなるパスにコピーする
- `match "posts/*"` の後にversionを利用してOGP画像のバージョンを作成する
    - `route $ setExtension "png"` のようにしてPNG画像のパスを作成する

こんなところでしょうか？

こんな流れで生成した画像を対象のパスにコピーし、生成完了となりました。

## 実装を終えて
面倒でした。

色々躓きポイントが多かったです。
例えばdiagramsがPNG画像の生成にcairoを使っているのですが、cabalの依存関係解決の時にこの辺のライブラリがないと落ちるので、最初のうちはcabalの問題かと勘違いして時間を浪費してしまいました。

GitHub Actionsでビルドする例は [.github/workflows/deploy.yaml](https://github.com/cordx56/blog.cordx.cx/blob/main/.github/workflows/deploy.yaml) にあるので、これも記載しておきます。

以上、HakyllでOGP画像を生成する話でした。
