# erbfmt VSCode Extension

Rust製 `erbfmt` binary を呼び出す薄いVSCode wrapperです。

## 挙動

- `*.html.erb` 向けに `html-erb` language id を提供します。
- `erb` と `html-erb` の document formatter を登録します。
- open / save 時に `erbfmt --lint` を実行し、diagnostics を表示します。
- 設定された `erbfmt` command を呼び出し、stdout を整形結果としてdocumentへ反映します。
- 整形ロジックはRust binary側に保持します。

## ローカル開発

このリポジトリから VSCode の Extension Development Host を使います。

1. 先に `cargo build` を一度実行します。
2. VSCodeでリポジトリルートを開きます。
3. Run and Debug view を開きます。
4. `Run erbfmt VSCode Extension` を選びます。
5. F5 を押します。
6. 新しく開いた Extension Development Host で
   `samples/sample.html.erb` を開きます。
7. `erbfmt: Format Document` を実行します。

`samples/sample.html.erb` は意図的に未整形です。extension が動いていれば、
`erbfmt: Format Document` の実行でインデントが変わります。
VSCode標準の `Format Document` も、erbfmt が default formatter として選ばれていれば動くはずです。
動かない場合は `Format Document With...` から `erbfmt` を選んでください。

このリポジトリの workspace 設定では、extension がローカルのRust checkoutを呼び出します。

```json
{
  "erbfmt.command": "cargo",
  "erbfmt.arguments": ["run", "--quiet", "--"]
}
```

このcheckoutから実行している場合、extension は `target/debug/erbfmt` があればそれを使います。
まだbinaryがない場合は `cargo run --quiet --` にfallbackします。

`erbfmt.command` には実行ファイルだけを指定します。追加のcommand-line argumentsは
`erbfmt.arguments` に分けて指定してください。

代わりに、先にRust binaryをインストールして使うこともできます。

```bash
cargo install --path ../..
```

特定の `erbfmt.json` を使う場合は `erbfmt.configPath` を設定します。
未指定の場合、extension は整形対象ファイルからfilesystem rootへ向かって
`erbfmt.json` を探します。

diagnostics を無効にする場合は `erbfmt.lint.enabled` を `false` に設定します。
