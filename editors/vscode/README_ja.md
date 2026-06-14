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
2. `npm install --prefix editors/vscode` を一度実行します。
3. VSCodeでリポジトリルートを開きます。
4. Run and Debug view を開きます。
5. `Run erbfmt VSCode Extension` を選びます。
6. F5 を押します。
7. 新しく開いた Extension Development Host で
   `samples/sample.html.erb` を開きます。
8. `erbfmt: Format Document` を実行します。

F5 の launch configuration は Extension Development Host を起動する前に
`npm run compile --prefix editors/vscode` を実行します。

このリポジトリには nodenv 用の `.node-version` を含めています。
現在のローカルNode versionは `24.10.0` です。

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

## TypeScript

extension のsourceは `src/extension.ts` にあり、`out/extension.js` へcompileされます。

```bash
npm run check --prefix editors/vscode
npm run compile --prefix editors/vscode
npm test --prefix editors/vscode
```

extension code のformat / lint は Biome で行います。

```bash
npm run format --prefix editors/vscode
npm run lint --prefix editors/vscode
```

## ローカルPackage

ローカル用の VSIX package を作成します。

```bash
npm run package --prefix editors/vscode
```

生成された VSIX はリポジトリルートからインストールできます。

```bash
code --install-extension editors/vscode/erbfmt-vscode-0.1.0.vsix
```

現時点のpackageにはRust binaryを同梱していません。別途 `erbfmt` をインストールするか、
`erbfmt.command` でローカルbinaryを指定してください。
