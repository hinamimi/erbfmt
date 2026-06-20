# erbfmt

[English](README.md)

`*.html.erb` テンプレート向けのFormatter/Linterです。高速なRust CLIとして動作し、
HTMLとERB制御構文をまとめて整形します。安全に変更できないRuby codeは保持します。

> erbfmtは現在プレリリース開発中です。repositoryからCLIを利用できますが、
> 公開release binary、RubyGems package、VSCode Marketplace extensionはまだ
> 公開していません。

## インストール

現在のインストール方法ではRust toolchainが必要です。

```bash
cargo install --git https://github.com/hinamimi/erbfmt --locked
```

インストール後にcommandを確認します。

```bash
erbfmt --version
erbfmt --help
```

Linux、macOS、Windows向けのprebuilt binaryは最初の公開releaseで提供する予定です。
platform-specific Ruby gemとVSCode extensionはartifactとして検証済みですが、
package registryにはまだ公開していません。

## クイックスタート

Rails projectに `erbfmt.json` を作成します。

```bash
cd your-rails-project
erbfmt init
```

ファイルを整形して標準出力へ表示します。

```bash
erbfmt app/views/users/show.html.erb
```

複数のファイルへ整形結果を書き戻せます。

```bash
erbfmt --write app/views/users/show.html.erb app/views/users/edit.html.erb
```

ファイルを変更せず、整形済みか確認できます。CIでも利用できます。

```bash
erbfmt --check app/views/users/show.html.erb app/views/users/edit.html.erb
```

Linterを実行します。

```bash
erbfmt --lint app/views/users/show.html.erb
```

`--write`、`--check`、`--lint` は同時に指定できません。checkは整形による変更が
必要な場合、lintはerror levelの診断が見つかった場合にnonzero statusで終了します。

## 整形例

入力:

```erb
<div>
<% if user %>
<p>Hello, <%= user.name %></p>
<% else %>
<p>Please sign in.</p>
<% end %>
</div>
```

出力:

```erb
<div>
  <% if user %>
    <p>Hello, <%= user.name %></p>
  <% else %>
    <p>Please sign in.</p>
  <% end %>
</div>
```

デフォルトではHTMLの階層とERB制御構文の両方をインデントします。`elsif`、`else`、
`when`、`rescue`、`ensure` などの分岐や、`<%= form_with ... do |form| %>` のような
output do-blockも認識します。

長いHTML tagはattributeごとの複数行へ展開します。単独の単純なRuby command callは、
引数を安全に分割できる場合に限り、明示的な括弧を付けて折りたたむことがあります。
複雑または曖昧なRuby式はそのまま保持します。

## 設定

erbfmtはカレントディレクトリと親ディレクトリから `erbfmt.json` を検索します。
デフォルト設定は次のcommandで生成できます。

```bash
erbfmt init
```

設定ファイルを明示する場合は `--config` を使います。

```bash
erbfmt --config path/to/erbfmt.json app/views/users/show.html.erb
```

indent style/width、HTML indentation、line width、line ending、lint ruleごとのseverityを
設定できます。全項目は [Configuration](docs/Configuration.md)、診断内容は
[Lint Rules](docs/LintRules.md) を参照してください。

生成されたmarkupや外部由来のmarkupを保持する場合は `erbfmt-ignore` commentを使えます。
構文は [Ignore Directives](docs/Ignore.md) にまとめています。

## VSCode

first-party extensionは `html-erb` syntax highlighting、document formatting、
diagnostics、ERB向けcomment toggleを提供します。Marketplaceにはまだ公開していないため、
現在はlocal VSIXと利用可能な `erbfmt` commandが必要です。

現在のインストール方法とcommand解決は [VSCode Integration](docs/VSCode.md) を参照してください。

## 現在の制約

- Ruby codeを完全なRuby ASTとして解析しません。
- Rails applicationのsemantic analysisは行いません。
- 安全に認識できない式は積極的に書き換えず、元の形を保持します。
- `pre`、`textarea`、`script`、`style` などのpreformatted contentは安全側で保持します。
- 最初の公開releaseまではpackage registryやMarketplaceからインストールできません。

## ドキュメント

- [Configuration](docs/Configuration.md)
- [Lint Rules](docs/LintRules.md)
- [Ignore Directives](docs/Ignore.md)
- [VSCode Integration](docs/VSCode.md)
- [Development](docs/Development.md)
- [Roadmap](docs/Roadmap.md)
