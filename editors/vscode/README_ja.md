# erbfmt for VS Code

[English](https://github.com/hinamimi/erbfmt/blob/main/editors/vscode/README.md)

**VS Code上でERBとHTML+ERBをformatし、lintできます。**

```diff
-<div><% if user.admin? %><span>Admin</span><% end %></div>
+<div>
+  <% if user.admin? %>
+    <span>Admin</span>
+  <% end %>
+</div>
```

高速なRust製CLI [erbfmt](https://github.com/hinamimi/erbfmt) をVS Codeから利用する
ためのextensionです。formatとlintの処理はCLIに集約されるため、command line、CI、
editorで同じ結果を得られます。

> 現在のextensionは `erbfmt` binaryを同梱またはdownloadしません。formatする前に
> CLIを別途installするか、`erbfmt.command`を設定してください。

## 機能

- `*.html.erb` 向けのHTML/ERB syntax highlighting
- `html-erb` と `erb` language idのdocument formatting
- documentを開いたときと保存したときのerbfmt lint diagnostics
- ERBを安全に扱う `Ctrl+/` / `Cmd+/` comment toggle
- active fileを起点にした `erbfmt.json` の自動検出
- CLI、追加arguments、config pathの明示的な設定
- 解決されたcommandとworking directoryを確認する `erbfmt: Show Command`

## 必要なもの

[v0.1.4 GitHub Release](https://github.com/hinamimi/erbfmt/releases/tag/v0.1.4)から
CLIをdownloadしてinstallします。Rust toolchainがある場合はtagged sourceからも
installできます。

```bash
cargo install --git https://github.com/hinamimi/erbfmt --tag v0.1.4 --locked
erbfmt --version
```

releaseではprebuilt binaryとplatform-specific gem fileも提供します。
extensionは、実行可能な `erbfmt` commandを提供するいずれのinstall方法でも利用できます。

## Extensionのインストール

extensionはVS Code Marketplaceへ公開していません。
[`erbfmt-vscode-0.1.4.vsix`](https://github.com/hinamimi/erbfmt/releases/download/v0.1.4/erbfmt-vscode-0.1.4.vsix)
をdownloadしてlocal installします。

```bash
code --install-extension erbfmt-vscode-0.1.4.vsix
```

`*.html.erb`を開いて **Format Document** を実行します。formatterの選択を求められた
場合は **erbfmt** を選択してください。

保存時に自動でformatする場合:

```json
{
  "[html-erb]": {
    "editor.defaultFormatter": "erbfmt.erbfmt-vscode",
    "editor.formatOnSave": true
  }
}
```

## クイックスタート

Rails projectのrootにconfig fileを作成します。

```bash
erbfmt init
```

extensionはactive documentからfilesystem rootへ向かって `erbfmt.json`を検索します。
workspaceでconfig fileを明示する必要がある場合のみ `erbfmt.configPath`を使います。

lint diagnosticsはdefaultで有効で、ERB documentを開いたときと保存したときに更新されます。
手動実行には **erbfmt: Lint Document** を使います。

## Bundlerで使う

erbfmtをRuby gemとして導入したprojectでは、bundle内のversionを実行できます。

```json
{
  "erbfmt.command": "bundle",
  "erbfmt.arguments": ["exec", "erbfmt"]
}
```

commandはactive documentのdirectoryから実行されるため、そのdirectoryまたは親directoryの
`Gemfile`をBundlerが見つけられます。

## Commands

| Command | 用途 |
| --- | --- |
| `erbfmt: Format Document` | active ERB documentをformatします。 |
| `erbfmt: Lint Document` | lint diagnosticsを更新します。 |
| `erbfmt: Show Command` | executable、arguments、cwd、configを表示します。 |
| `erbfmt: Toggle Comment` | 選択範囲のERB-safe commentをtoggleします。 |

## Settings

| Setting | Default | 用途 |
| --- | --- | --- |
| `erbfmt.command` | `erbfmt` | erbfmtを実行するexecutableです。 |
| `erbfmt.arguments` | `[]` | erbfmt固有のargumentsより前に追加します。 |
| `erbfmt.configPath` | empty | 特定の `erbfmt.json`を指定します。 |
| `erbfmt.lint.enabled` | `true` | open/save時にdiagnosticsを表示します。 |

`erbfmt.command`にはexecutableだけを設定します。たとえばcommandを `bundle`、
`erbfmt.arguments`を `exec`, `erbfmt`とします。

## コメント

`Ctrl+/`または `Cmd+/`は行単位でcommentをtoggleします。ERB tagは
`<%# if user %>`や `<%#= user.name %>`のようなERB commentになります。
HTML fragmentはHTML commentになります。HTMLとERBが混ざる行は、ERB codeが
HTML comment内で誤って実行されないように分割します。

## Troubleshooting

formatまたはdiagnosticsが `ENOENT`や `EACCES`で失敗する場合:

1. terminalで `erbfmt --version`が動くことを確認します。
2. **erbfmt: Show Command** でexecutableとworking directoryを確認します。
3. VS Codeからterminalと同じ `PATH`が見えない場合は、`erbfmt.command`に実行可能な
   absolute pathを設定します。
4. commandのargumentsは `erbfmt.command`ではなく `erbfmt.arguments`に設定します。

Shopify Ruby LSPと併用できます。このextensionは `html-erb` language idを提供し、
`html-erb`と `erb`の両方にformatterを登録します。

## 開発

repository rootから実行します。

```bash
cargo build
npm install --prefix editors/vscode
npm test --prefix editors/vscode
npm run package --prefix editors/vscode
```

repositoryをVS Codeで開き、**Run erbfmt VSCode Extension** を選んでF5を押すと、
Extension Development Hostを起動できます。extension-host test、command解決、release
packageの詳細は
[VSCode integration documentation](https://github.com/hinamimi/erbfmt/blob/main/docs/VSCode.md)
を参照してください。
