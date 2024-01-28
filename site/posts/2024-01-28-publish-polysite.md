---
title: Rustで静的サイトジェネレータライブラリを作った
date: 2024-01-28 19:42
tags: [rust, polysite]
---

こんにちは。
修論締切約一週間前です。
そんな時期になんでこんなもの書いてるのかって？
なんででしょう……

## Rust製静的サイトジェネレータ
今回の話題は、静的サイトジェネレータとRustです。
Rust製静的サイトジェネレータとしては[Zola](https://www.getzola.org/)が非常に有名ですね。
ですが、表題の通り、今回は「静的サイトジェネレータライブラリを作」りました。
なぜ作ったのか、どのようにした作ったのかなどの話を書きたいと思います。
ちなみにこのサイトもそのライブラリを利用してビルドしています。

### 静的サイトジェネレータライブラリ「[polysite](https://crates.io/crates/polysite)」
今回作成した静的サイトジェネレータライブラリの名前は「polysite」です。
リンクでお気づきかもしれませんが、[Crates.io](https://crates.io/)で公開しています。

名前に「poly」と入れたのは、多様な静的サイト生成の需要に応えたいという思いがあります。
そもそも私がZolaを使わなかった理由に触れつつ、このライブラリの作成経緯について話していきます。

### 既存の静的サイトジェネレータを使わなかった理由
ZolaはRust製で、簡単に動く軽量な静的サイトジェネレータとして非常に優秀だと思います。
依存関係もRustのみで、シングルバイナリで動くという大きな利点があります。
ですが、私には一つ、致命的にZolaを使えない理由がありました。
それが、カスタマイズ性の低さです。

Zolaは全部入りで簡単に使える静的サイトジェネレータとして優秀ですが、ジェネレータ自体のカスタマイズをすることは前提にされていません。
[Zolaのリポジトリ](https://github.com/getzola/zola)の機能一覧を見ても、特に内部のカスタマイズについては言及されていません。

私はもともと[Hakyll](https://jaspervdj.be/hakyll/)を使って静的サイト生成を行っていて、Hakyllデフォルトの機能に加え、OGP用の画像の生成を行うカスタムをしていました。
Haskellは普段あまり書かなくてベストプラクティスを知らず、汚いコードですが、[実際にOGP画像を生成しているコード](https://github.com/cordx56/blog.cordx.cx/blob/hakyll-last/app/Image.hs)もあります。
OGP画像の生成は私にとっては必須条件と言っても過言ではありませんでした。
だって個人ブログでも綺麗なOGP画像が表示されて欲しいじゃないですか？

以前のままHakyllを使ってHaskellを書き続けてもよかったのですが（Pandocもあることですし）、HaskellでOGP画像の生成があまり綺麗に行えない問題がありました。
問題があれば自力で解決がエンジニアとしてあるべき姿かもしれませんが、私は画像処理もフォントも文字描画もHaskellも全て門外漢です。
この状況から静的サイトジェネレータのために全てを勉強するのはあまりにも過酷です。

あとおそらく当分解決不可能な問題に、Haskellのビルド時間が遅いという問題がありました。
Hakyllを使って書いた静的サイトジェネレータのビルドには、GitHub Actionsで20分以上かかります。
Rustだったらもっと速くビルドできるだろうな、という感覚があります。

### Rustで静的サイトジェネレータライブラリを探す
まだ私が書けそうな言語でHaskell以上にライブラリが充実している、そして言語や周辺エコシステムが使いやすいものとなると、RustかPythonしか思いつきませんでした。
Pythonの静的サイトジェネレータだと[Pelican](https://getpelican.com/)などが有名ですが、私は以前使ってみて、Hakyllほど便利ではなく使うのをやめた記憶があります（今はまだマシかもしれません、またプラグインシステムなどもあるようですね）。
個人的にはRustで書けると嬉しいな〜というのがあり、Crates.ioを探しましたが、他の静的サイトジェネレータライブラリでも、私が求めるカスタマイズ性を備えていてドキュメントが充実していて……みたいなものを見つけることができませんでした。

### Rustで静的サイトジェネレータライブラリを作る
これらの事情から、Rustで静的サイトジェネレータを作ることにしました。
静的サイトジェネレータに求めるものは、次のようになります。

- Hakyllのように抽象化され、Markdownの静的サイトを簡単にビルドできる機構
- 高いカスタマイズ性
    - 最悪ビルドプロセス中に生のRustをかけるようなレベルのカスタマイズ性
- 私の今のブログサイトが問題なくビルドできること

これらを満たすものをRustで作ることが、今回の静的サイトジェネレータ作りの最終目標になりました。

### できたもの
上記の結果、できたものがpolysiteになります。

polysiteの依存関係を見てもらうとわかると思いますが、テンプレートエンジンに[Tera](https://crates.io/crates/tera)、マークダウンのレンダーに[pulldown-cmark](https://crates.io/crates/pulldown-cmark)と、依存している技術の構成はZolaとほとんど同じです。

#### ビルドプロセス
polysiteでビルドを行うプロセスは次のようになります。

1. [`Config`][Config] を引数に [`Builder`][Builder] を作成する
2. コンパイルするファイルの指定やバージョンの指定を行う [`Rule`][Rule] を作成する
3. [`Builder`][Builder] にステップとして複数の [`Rule`][Rule] を追加する
    - 同じステップに登録された [`Rule`][Rule] は concurrently にコンパイルされる
4. [`Builder`][Builder] 型の [`build`][build] メソッドを呼び出し、 `await` する。

さらに、 [`Rule`][Rule] の処理は次のようになっています。

1. 指定されたglobなどからファイル一覧を取ってくる
2. 取ってきたファイル一覧などの情報から [`Context`][Context] と [`Metadata`][Metadata] を準備する
3. 指定されたコンパイラを利用して各ファイルをコンパイルする

各ファイルを処理するコンパイラは [`Compiler`][Compiler] トレイトを実装した型であり、 [`Rule`][Rule] は [`Compiler`][Compiler] の [`compile`][compile] メソッドを各ファイルに対して実行します。

各コンパイラは [`Context`][Context] にある [`Metadata`][Metadata] の情報を利用してコンパイルを行い、コンパイル結果を `anyhow::Result<Context>` として返します（厳密にはここに `Box<dyn Future<...` などが返り値の型として関わってきます）。
例としてソースとなるファイルを読み込んでマークダウンをパースしてターゲットファイルに書き込むコンパイラを書いてみます。

```
pipe!(
     SetExtension::new("html"),
     FileReader::new(),
     MarkdownRenderer::new(None),
     FileWriter::new(),
)
```

順を追ってみていきます。
まず、 [`pipe!`][pipe] マクロがあります。
このマクロは、複数のコンパイラを連結して大きなコンパイラを作るために使われます。
連結されている各コンパイラは次のように処理を行います。

1. [`SetExtension`][SetExtension] コンパイラが出力先ファイルの拡張子 [`Metadata`][Metadata] として設定
2. [`FileReader`][FileReader] コンパイラがファイルの内容を読み込み、 [`Metadata`][Metadata] にファイル内容を記録
3. [`MarkdownRenderer`][MarkdownRenderer] コンパイラが [`Metadata`][Metadata] からファイル内容を読み込み、HTMLをレンダーしてファイル内容として記録
4. [`FileWriter`][FileWriter] コンパイラが [`Metadata`][Metadata] からファイル内容を読み込み、出力先ファイルに書き込み

これが一つのファイルのコンパイル手順になります。
これを [`Rule`][Rule] が複数のファイルに対して適用し、全てのファイルをコンパイルします。

実際の例は、リポジトリの [`examples`](https://github.com/cordx56/polysite/tree/main/examples) や [このサイトのビルドスクリプト](https://github.com/cordx56/blog.cordx.cx/tree/7a93a1249f21dd23d8c55dad7e7bffbfe1213673/src) を見るとわかると思います。

[Config]: https://docs.rs/polysite/0.0.1/polysite/config/struct.Config.html
[Builder]: https://docs.rs/polysite/0.0.1/polysite/builder/builder/struct.Builder.html
[Rule]: https://docs.rs/polysite/0.0.1/polysite/builder/rule/struct.Rule.html
[build]: https://docs.rs/polysite/0.0.1/polysite/builder/builder/struct.Builder.html#method.build
[Context]: https://docs.rs/polysite/0.0.1/polysite/builder/context/struct.Context.html
[Metadata]: https://docs.rs/polysite/0.0.1/polysite/builder/metadata/type.Metadata.html
[Compiler]: https://docs.rs/polysite/0.0.1/polysite/compiler/trait.Compiler.html
[compile]: https://docs.rs/polysite/0.0.1/polysite/compiler/trait.Compiler.html#tymethod.compile
[pipe]: https://docs.rs/polysite/0.0.1/polysite/macro.pipe.html
[SetExtension]: https://docs.rs/polysite/0.0.1/polysite/compiler/path/struct.SetExtension.html
[FileReader]: https://docs.rs/polysite/0.0.1/polysite/compiler/file/struct.FileReader.html
[MarkdownRenderer]: https://docs.rs/polysite/0.0.1/polysite/compiler/markdown/struct.MarkdownRenderer.html
[FileWriter]: https://docs.rs/polysite/0.0.1/polysite/compiler/file/struct.FileWriter.html

#### 生成結果
polysiteを利用した静的サイトジェネレータでは、私の既存のサイトをほとんど問題なくビルドできました。
ただ一点、Pandocでは脚注がいい感じに出力されていましたが、pulldown-cmarkでは少し修正が必要になりました。

OGP画像は[imageproc](https://crates.io/crates/imageproc)を使うことで、綺麗にレンダーされています。

#### ビルド時間
Haskellでは同じサイトをビルドするために、プログラムのコンパイル時間も含めて20分かかっていましたが、Rustでは1分未満です。
速い。
速いです。
ブログなんて書いてpushしたら寝るだけなのでビルド時間の長さはそんなに気にすることもないのですが、それでもビルドがコケないということをすぐに確認できるということが下げる心理的な負担は大きいです。
あとビルド時間が十分に短いので、キャッシュの利用みたいな細工を考える必要もないです。

## Crates.ioへの公開
実はCrates.ioへパッケージを公開するのは初めてでした。
ドキドキですね。
私の書いたカスのプログラムがパッケージとして全世界に公開されてしまうことを思うと、なんだか変な気持ちになります。
おえ、吐きそう。

どうでもいいことはさておき、crates.ioへの公開のためにやったことを書いていきます。
他のパッケージリポジトリと大体同じようなものです。

- ドキュメントの充実
    - 当然のことですね。Rustの綺麗なドキュメントが[docs.rs](https://docs.rs/)でホストされることを考えると多少やる気が出ます。
- `Cargo.toml` への情報の記載
    - これは[デフォルトの `Cargo.toml` に記載されているリンク](https://doc.rust-lang.org/cargo/reference/manifest.html)を参考にしながら書くのが良いでしょう
- `README.md` や `LICENSE` を置く
- `cargo login`
- `cargo publish`

最後の公開に向けた確認は[Crates.ioにクレートを公開する](https://doc.rust-jp.rs/book-ja/ch14-02-publishing-to-crates-io.html)の記事を見ながら行いました。

ちなみに公開されたCrates.ioのREADME.mdを見ていただければわかると思いますが、マークダウンをミスってます。
普段は間違えないんですが、ドキュメントに書いたマークダウンの一部をREADME.mdに移す……みたいな作業をしたら事故りました。
だから言わんこっちゃない。
学位論文の時期は学位論文に集中しましょう。

## さいごに
polysite、もし機会があればお使いください。
まだ荒削りなので、あなたのPRをお待ちしております。
今後は、GitHubにもソースコードにもドキュメントを増やし、使いやすい静的サイトジェネレータライブラリを目指していきたいです。

あと、学位論文が忙しい時期にパッケージリポジトリにパッケージを公開するのは（脆弱性対応とかを除き）やめておきましょう。
