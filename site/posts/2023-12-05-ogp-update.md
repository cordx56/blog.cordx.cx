---
title: OGP画像の生成を新しくした
date: 2023-12-05 11:39
tags: [haskell]
---

こんにちは。

以前HaskellでOGP画像を自動生成するという話を書いたのですが、より良い方法があったのでそれにした、という話です。

![OGP画像のスクリーンショット](/imgs/2023-12-05.png)

SVGFontsを用いたやり方では、この画像のように一部の文字が欠落してしまいました。
理由はSVGフォントのUnicodeのエイリアス周りが壊れているようでした。

## Rasterificを使う
その後、なんとなくいいライブラリがないか探していたところ、 [Rasterific](https://hackage.haskell.org/package/Rasterific) というライブラリを見つけました。
このライブラリは [JuicyPixels](https://hackage.haskell.org/package/JuicyPixels) と [FontyFruity](https://hackage.haskell.org/package/FontyFruity) を利用して、文字の描画ができるようです。
さらに、外部のライブラリに依存してないっぽいので、非常に便利です。
ので、これを使うことにしました。

## 折り返しとか
文字の折り返しなどは既存のものはなかったので、自分で実装しました。
[app/Image.hs](https://github.com/cordx56/blog.cordx.cx/blob/main/app/Image.hs) とかに書いてありますが、一文字ずつ取ってきて横幅を計算して……みたいなことをしています。

## 所感
以前に比べすっきりとした実装・手法になったように感じます。
特に、依存関係がすっきりしたのが嬉しいですね。
