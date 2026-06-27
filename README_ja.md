# erbfmt

[English](README.md)

**ERBとHTML+ERB向けの高速な、Prettier/BiomeのようなFormatter/Linterです。**

```diff
-<div><% if user.admin? %><span>Admin</span><% end %></div>
+<div>
+  <% if user.admin? %>
+    <span>Admin</span>
+  <% end %>
+</div>
```

erbfmtはHTML構造とERB制御構文をまとめて整形し、安全に書き換えられないRuby式は
保持します。Railsの `*.html.erb` template向けのRust CLIとして、local、CI、
first-party VSCode extensionから利用できます。

> [!WARNING]
> erbfmtはbeta版です。stable releaseまでの間、format結果、設定、lint rule、CLIの動作が
> 後方互換性なく変更される可能性があります。format差分を確認してからcommitし、
> 自動実行する環境ではversionを固定してください。

> erbfmtは現在プレリリース開発中です。Version `0.1.4`をGitHub Releasesで
> 公開しています。初期releaseはpackage indexやextension marketplaceへ登録しません。

## インストール

[v0.1.4 release](https://github.com/hinamimi/erbfmt/releases/tag/v0.1.4)から
利用するplatformのarchiveをdownloadして展開し、`erbfmt`または `erbfmt.exe`を
`PATH`へ配置します。

- Linux x64: `x86_64-unknown-linux-gnu`
- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows x64: `x86_64-pc-windows-msvc`

Rust toolchainがある場合は、tagged sourceをGitHubから直接installできます。

```bash
cargo install --git https://github.com/hinamimi/erbfmt --tag v0.1.4 --locked
```

インストール後にcommandを確認します。

```bash
erbfmt --version
erbfmt --help
```

releaseにはplatform-specific `.gem` fileとVSIXも含まれます。crates.io、npm、
RubyGems.orgには公開していないため、registry経由のinstall commandは利用できません。

Rails projectのGemfileでerbfmtを管理する場合は、利用platformに対応するgemを
downloadし、`vendor/gems`へ展開してpath gemとして参照します。`gem unpack`だけでは
Bundlerが読む `.gemspec` が展開されないため、`gem spec --ruby` も実行します。

```bash
curl -L \
  -o erbfmt-0.1.4-x86_64-linux-gnu.gem \
  https://github.com/hinamimi/erbfmt/releases/download/v0.1.4/erbfmt-0.1.4-x86_64-linux-gnu.gem
mkdir -p vendor/gems
gem unpack erbfmt-0.1.4-x86_64-linux-gnu.gem --target vendor/gems
gem spec erbfmt-0.1.4-x86_64-linux-gnu.gem --ruby \
  > vendor/gems/erbfmt-0.1.4-x86_64-linux-gnu/erbfmt.gemspec
```

```ruby
group :development do
  gem "erbfmt",
    path: "vendor/gems/erbfmt-0.1.4-x86_64-linux-gnu",
    require: false
end
```

```bash
bundle install
bundle exec erbfmt --version
```

開発環境ごとに対応するplatform gemを使ってください。platform名、offline install、
複数platformのprojectについては
[Ruby Gem Wrapper](docs/RubyGem.md#installing-from-a-gemfile)にまとめています。

## クイックスタート

Rails projectに `erbfmt.json` を作成します。

```bash
cd your-rails-project
erbfmt init
```

ファイルを整形して書き戻します。

```bash
erbfmt --write app/views/users/show.html.erb
```

複数のファイルもまとめて整形できます。

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

## erbfmtが扱う範囲

デフォルトではHTMLの階層とERB制御構文の両方をインデントします。`elsif`、`else`、
`when`、`rescue`、`ensure` などの分岐や、`<%= form_with ... do |form| %>` のような
output do-blockも認識します。

長いHTML tagはattributeごとの複数行へ展開します。明示的な括弧の有無にかかわらず、
単独の単純なRuby method callは、引数を安全に分割できる場合に限り、引数ごとの複数行へ
折りたたむことがあります。複雑または曖昧なRuby式はそのまま保持します。

whitespaceの意味が変わりやすいinline outputは安全側に倒します。隣接するinline HTML、
隣接するERB output、元ソースで1行だったERB blockは、`formatter.lineWidth`を超えても
inlineのまま保持します。`pre`、`textarea`、`script`、`style`、`svg`、`math`、
`contenteditable` やinline `white-space` styleを持つelementのsubtreeも保持します。
`template` と `noscript` のsubtreeも折り返さず保持します。ただし、保持対象の
opening tagは、contentを変えずに安全に扱える場合、attributeごとに整形することがあります。

Linterは不正なHTML構造、list/tableの不正なnesting、非推奨tag、self-closing tag、
重複attribute、未対応または空のERB制御構文を検出します。

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

`formatter.trailingNewline` はdefaultで `true` です。通常のtemplate fileではこの設定を
推奨します。周囲のtextへinline partialとして差し込むERB fileで、末尾newlineを
render結果へ含めたくない場合は、そのfileまたはprojectで `false` にしてください。

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
- `<%-`、`-%>`、`<%%`、`<%==`などのstandalone ERB trim、escaped、raw-output
  markerは、安全でない書き換えを避けるため現在はerrorにします。
- 安全に認識できない式は積極的に書き換えず、元の形を保持します。
- `pre`、`textarea`、`script`、`style`、`svg`、`math`、`template`、`noscript`、
  `contenteditable` subtree、inline `white-space` styleなどのpreformatted /
  format-sensitive contentは安全側で保持します。
- 初期releaseはGitHub Releasesだけで配布し、package registryやextension marketplace
  には登録しません。

## ドキュメント

- [ドキュメントサイト](https://hinamimi.github.io/erbfmt/ja/)
- [Configuration](docs/Configuration.md)
- [Lint Rules](docs/LintRules.md)
- [Ignore Directives](docs/Ignore.md)
- [VSCode Integration](docs/VSCode.md)
- [Development](docs/Development.md)
- [Roadmap](docs/Roadmap.md)
