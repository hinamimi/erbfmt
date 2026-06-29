# erbfmt for VS Code

[English](https://github.com/hinamimi/erbfmt/blob/main/editors/vscode/README.md)

**VS Code上でERBとHTML+ERB templateをformat、lint、highlight、comment toggleできます。**

```diff
-<div><% if user.admin? %><span>Admin</span><% end %></div>
+<div>
+  <% if user.admin? %>
+    <span>Admin</span>
+  <% end %>
+</div>
```

高速なRust製CLI [erbfmt](https://github.com/hinamimi/erbfmt) をVS Codeから利用する
ためのextensionです。Railsの `*.html.erb` template向けに、formatとlintの処理はCLIに
集約されるため、command line、CI、editorで同じ結果を得られます。

> [!IMPORTANT]
> 現在のextensionは `erbfmt` binaryを同梱またはdownloadしません。formatする前に
> CLIを別途installするか、`bundle exec erbfmt` のようなproject-local commandを
> `erbfmt.command` に設定してください。

> [!WARNING]
> erbfmtはbeta版です。format差分を確認してからcommitし、自動実行する環境では
> CLI versionを固定してください。

## 機能

- `*.html.erb` 向けのHTML/ERB syntax highlighting
- `html-erb` と `erb` language idのdocument formatting
- documentを開いたときと保存したときのerbfmt lint diagnostics
- erbfmtの出力と差がある行をwarnするformat diagnostics
- ERBを安全に扱う `Ctrl+/` / `Cmd+/` comment toggle
- active fileを起点にした `erbfmt.json` の自動検出
- CLI、追加arguments、config pathの明示的な設定
- 解決されたcommandとworking directoryを確認する `erbfmt: Show Command`

## インストール

VS Code Marketplaceから **erbfmt** をinstallし、その後 erbfmt CLI を以下のいずれかの
方法でinstallします。

### RubyGems / Bundler

Rails projectではBundlerでerbfmtを固定する方法が便利です。

```bash
bundle add erbfmt --group development --require false
bundle exec erbfmt --version
```

extensionにprojectで固定したcommandを使うよう設定します。

```json
{
  "erbfmt.command": "bundle exec erbfmt"
}
```

手元でglobal commandとして使いたい場合は、RubyGemを直接installします。

```bash
gem install erbfmt -v 0.2.0
erbfmt --version
```

global installはdefault設定の `"erbfmt.command": "erbfmt"` で動きます。projectでは
formatter versionを固定できるBundlerを優先してください。

### GitHub Release Binary

[v0.2.0 GitHub Release](https://github.com/hinamimi/erbfmt/releases/tag/v0.2.0)から
CLIをdownloadして展開し、`erbfmt` または `erbfmt.exe` を `PATH` に配置します。

### Cargo

Rust toolchainがある場合はtagged sourceからinstallできます。

```bash
cargo install --git https://github.com/hinamimi/erbfmt --tag v0.2.0 --locked
erbfmt --version
```

## クイックスタート

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

Rails projectのrootにconfig fileを作成します。

```bash
erbfmt init
```

extensionはactive documentからfilesystem rootへ向かって `erbfmt.json`を検索します。
workspaceでconfig fileを明示する必要がある場合のみ `erbfmt.configPath`を使います。

diagnosticsはdefaultで有効で、ERB documentを開いたときと保存したときに更新されます。
lint diagnosticsは `erbfmt --lint` から生成し、format diagnosticsはerbfmtの出力と差が
ある行をwarningとして表示します。手動実行には **erbfmt: Lint Document** を使います。

## Bundlerで使う

erbfmtをRuby gemとして導入したprojectでは、projectで固定したversionを実行できます。

```json
{
  "erbfmt.command": "bundle exec erbfmt"
}
```

extensionはこのcommandをshellを使わずにexecutableとargumentsへ分解し、workspaceまたは
active documentのdirectoryから実行します。そのdirectoryまたは親directoryの `Gemfile`を
Bundlerが見つけられます。

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
| `erbfmt.command` | `erbfmt` | `bundle exec erbfmt` など、erbfmtを実行するcommandです。 |
| `erbfmt.arguments` | `[]` | `erbfmt.command` の後ろに追加するargumentsです。 |
| `erbfmt.configPath` | empty | 特定の `erbfmt.json`を指定します。 |
| `erbfmt.lint.enabled` | `true` | open/save時にdiagnosticsを表示します。 |
| `erbfmt.formatDiagnostics.enabled` | `true` | documentが未formatの場合にwarningを表示します。 |

file pathより前に常に渡したい追加flagがある場合は `erbfmt.arguments`を使います。
たとえば `"erbfmt.command": "bundle exec erbfmt"` と
`"erbfmt.arguments": ["--some-flag"]` を組み合わせられます。

## コメント

`Ctrl+/`または `Cmd+/`は行単位でcommentをtoggleします。ERB tagは
`<%# if user %>`や `<%#= user.name %>`のようなERB commentになります。
HTML fragmentはHTML commentになります。HTMLとERBが混ざる行は、ERB codeが
HTML comment内で誤って実行されないように分割します。

## Troubleshooting

formatまたはdiagnosticsが `ENOENT`や `EACCES`で失敗する場合:

1. terminalで `erbfmt --version`が動くことを確認します。
2. **erbfmt: Show Command** でexecutableとworking directoryを確認します。
3. VS Codeからterminalと同じ `PATH`が見えない場合は、`erbfmt.command`に
   `bundle exec erbfmt` または実行可能なabsolute pathを設定します。
4. **erbfmt: Show Command** で `erbfmt.command` がどう分解されたか確認します。

Shopify Ruby LSPと併用できます。このextensionは `html-erb` language idを提供し、
`html-erb`と `erb`の両方にformatterを登録します。

## Links

- [erbfmt documentation](https://hinamimi.github.io/erbfmt/)
- [CLI repository](https://github.com/hinamimi/erbfmt)
- [Issues](https://github.com/hinamimi/erbfmt/issues)
- [Release notes](https://github.com/hinamimi/erbfmt/releases)

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
