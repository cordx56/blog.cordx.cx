---
title: Cloudflare WorkersでRust製wasmをimportして使う
date: 2024-06-11 22:19
tags: [cloudflare, rust, wasm]
---

こんにちは。
最近は本当に忙しい時期を脱して少し余裕ができました。
本当に？
わからない……

## Cloudflare Workersとwasm

Cloudflare Workersでwasmを使えるのは周知の事実だと思います。
特にCloudflareもRustでのCloudflare Workersの利用を[手厚くサポート](https://developers.cloudflare.com/workers/languages/rust/)しています。

上記ドキュメントを読めば、すべてRustで記述されたのCloudflare Workersは簡単にデプロイできます。
そもそもRustを使いたいという人は大体すべてRustで書くよ！という人が多いと思うので、まぁこれで十分という気もします。
ですが、TypeScriptで記述されたCloudflare Workersからwasmをimportして使う方法は、調べても全然出てきませんでした。
単純なWebアプリケーション開発をする場合はTypeScriptで十分ですし、Web系で広く使われていてライブラリも充実していて、wasm対応のために特別何かということもありません（Cloudflare WorkersではNode.js関連の問題は起きるかもしれませんね）。
一方、こうしたアプリケーションの内部でwasmを利用したいこともあると思います。

今回は、TypeScriptで記述されたCloudflare Workersアプリケーションの内部で、Rustで記述され（wasm-packでビルドされ）たwasmを利用する方法について、少し詰まったので書いておきます。

マジで時間がないので雑な内容ですがお許しください。

## 前提

ここでは前提について軽く説明します。

### wasmをwasm-packでビルドする

まぁこの記事を読んでいる方は大体わかっていると思うので、wasm-packを利用してwasmプロジェクトを作るところまでは省略します。
wasm-packを用いたビルドは以下のコマンドで行えますね。

```bash
wasm-pack build
```

これで、bundler向けのwasmがビルドされ、`pkg`ディレクトリに格納されます。

### wasmをTypeScriptからimportして使えない

こうしてできた`pkg`ディレクトリの中にあるjsファイルをCloudflare Workersで動かしたいTypeScriptからimportすれば動くのかな〜と思いきや、動きませんでした。

エラーを`console.log`してみると、`TypeError: wasm.__wbindgen_add_to_stack_pointer is not a function`というエラーが出てきます。
このエラーから、おそらくwasmが読み込めていないのだろうな〜という予想を立てました。

## Cloudflare Workersでwasmを読み込む

ここからは、実際にCloudflare WorkersにデプロイするTypeScriptプログラムからwasmを読み込んで利用する方法について書いていきます。

### wasmの読み込み方を探る

wasmが読み込めていないことがわかったので、wasmを読み込むために、Cloudflare Workers向けのRustのwasmビルドがどのようにして行われているか調べます。

[cloudflare/workers-rs](https://github.com/cloudflare/workers-rs)にある`worker-build`がRustのCloudflare Workers向けビルドを行うパッケージになっています。

この中を見てみると、[`worker-build/src/js/glue.js`](https://github.com/cloudflare/workers-rs/blob/749854f56f5839cf6c55a8dd6f594644f86263a4/worker-build/src/js/glue.js)がいかにもwasmの読み込みを行っている雰囲気があります。
この内容から、Cloudflare Workersでは、wasmの読み込みはwasm-packが吐き出すwasmを単にimportするコードでは足りず、`WebAssembly.Instance`を用いてwasmをインスタンス化する必要がありそうなことがわかります。

### wasm-packの出力したjsファイルを書き換える

以上のことから、wasm-packが出力したコードに対して、wasmの読み込み部分に手を加えてあげれば良さそうということがわかりました。
wasm-packの出力したwasm読み込みを行うJavaScriptコードを以下に示します。

```javascript
import * as wasm from "./image_conversion_bg.wasm";
import { __wbg_set_wasm } from "./image_conversion_bg.js";
__wbg_set_wasm(wasm);
export * from "./image_conversion_bg.js";
```

なお、今回のwasmプロジェクトはCloudflare Workers上で画像の変換を行うプロジェクトだったため、プロジェクト名を`image-conversion`としてあり、それがソースコード中に現れています。

wasm-packの出力したコードすべてを読むと（ここでは紹介しません！）、`__wbg_set_wasm(wasm)`が読み込んだwasmの関数をwasm用の変数に格納して利用できるようにする関数であることがわかります。
このコードでは`import * as wasm from "./image_conversion_bg.wasm";`がwasmファイルの読み込みを行っていますが、先ほど述べたように、Cloudflare Workersでは`WebAssembly.Instance`を利用してwasmを読み込む必要があります。

[`WebAssembly.Instance`のドキュメント](https://developer.mozilla.org/ja/docs/WebAssembly/JavaScript_interface/Instance/Instance)を読みつつ、上記プログラムを以下のように書き換えます。

```javascript
import wasm from "./image_conversion_bg.wasm";
import * as imports from "./image_conversion_bg.js";
const instance = new WebAssembly.Instance(wasm, { "./image_conversion_bg.js": imports });
imports.__wbg_set_wasm(instance.exports);
export * from "./image_conversion_bg.js";
```

重要なのは、

- wasmファイルを`wasm`という名前でimportする
- wasm-packが出力した`_bg.js`ファイルのexportすべてを`imports`という名前でimportする
- `WebAssembly.Instance`の第一引数に前で読み込んだwasmを、第二引数に`_bg.js`ファイルでimportしたものを渡す
- 得られた`instance.exports`を`imports.__wbg_set_wasm`に渡してwasmのexportしている関数を利用できるようにする

という点です。

これらの変更を加えたものをCloudflare Workersにデプロイすると、無事にwasmが読み込まれ、wasmの関数を利用することができるようになりました。

## おわりに

今回はCloudflare WorkersにデプロイするTypeScriptプログラム中で、wasm-packを利用してビルドされたwasmをimportするやり方について説明しました。

ちなみに画像の変換は、Cloudflare WorkersのCPU時間の制約でできませんでした（それはそう）。

~~大学の退館時間が迫っているので~~、よく内容確認してないですが多分大丈夫です。
退館時間を過ぎて違法滞在している……
問題があったらご連絡ください。
