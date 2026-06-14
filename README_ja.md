# erbfmt

Ruby ERBテンプレート向けの高速なFormatter/Linterです。

## 目的

ERBをTSXやHTMLと同じように整形できるツールを作ることです。

目標としている体験は、Prettier、Biome、dprint のような保存時自動整形です。

## 設計方針

ERB内のRubyコードを完全に理解することは目指しません。

まずは以下を実現します。

- HTML構造を正しく整形する
- ERB制御構文によるインデントを適用する
- Ruby式は基本的にそのまま保持する

### 例

```erb
<% if user %>
...
<% end %>
```

上記のようなERB制御構文はインデント対象とします。

## 現在の状態

MVP開発中

### 実装済み

- CLI
- ファイル読み込み
- Token定義
- ERB Lexer
- 軽量HTML Tokenizer
- HTML awareな混合Parser
- AST Parser
- 混合ASTベースのFormatter
- ERB制御構文のインデント
- `else`、`elsif`、`when`、`rescue`、`ensure` のERB分岐整形
- `case` / `when` ブロックの整形
- `<%= form_with ... do |form| %>` のような出力付きERB do-blockの整形
- HTMLタグ属性内のERB output
- HTMLタグ階層のインデント
- `--write` によるファイルの直接整形
- VSCode workspace の保存時整形設定
- Lexer / Parser / HTMLタグ対応診断を使った基本的なLinter
- 空のERBブロックと残りの未対応ERBブロック開始キーワード向けの構文lint
- `--check` による整形済みチェック
- ファイル名付きのCLI診断
- 複数ファイルのlint、check、write

## CLI

ローカルのcheckoutから `erbfmt` としてインストールできます。

```bash
cargo install --path .
```

インストールされたbinaryを確認します。

```bash
erbfmt --version
erbfmt --help
```

erbfmt は、カレントディレクトリまたは親ディレクトリにある `erbfmt.json` を読み込みます。
特定の設定ファイルを指定する場合は `--config` を使います。

```bash
erbfmt --config erbfmt.json samples/sample.html.erb
```

ファイルを整形します。

```bash
cargo run -- samples/sample.html.erb
erbfmt samples/sample.html.erb
```

ファイルへ直接書き戻す場合は `--write` を指定します。

```bash
cargo run -- --write samples/sample.html.erb
erbfmt --write samples/sample.html.erb
```

ファイルをlintする場合は `--lint` を指定します。

```bash
cargo run -- --lint samples/sample.html.erb
erbfmt --lint samples/sample.html.erb
```

ファイルが整形済みかどうかだけを確認する場合は `--check` を指定します。

```bash
cargo run -- --check samples/sample.html.erb
erbfmt --check samples/sample.html.erb
```

複数ファイルをlintまたはcheckすることもできます。

```bash
cargo run -- --lint samples/sample.html.erb samples/lint-next.html.erb
cargo run -- --check samples/sample.html.erb samples/lint-next.html.erb
```

`--write`、`--check`、`--lint` は同時に指定できません。`--no-html-indent` は整形やチェックでは使えますが、lintでは使えません。

デフォルトでは、ERB制御構文のネストとHTMLタグ階層の両方をインデントします。

例:

```erb
<div>
  <% if user %>
    <ul>
      <% Objects.map do |obj| %>
        <li><%= obj.name %></li>
      <% end %>
    </ul>
  <% elsif guest? %>
    <p>Guest</p>
  <% else %>
    <p>Please sign in</p>
  <% end %>
  <% case role %>
  <% when "admin" %>
    <p>Admin</p>
  <% when "user" %>
    <p>User</p>
  <% end %>
</div>
```

HTMLタグ階層によるインデントを無効にし、ERB制御構文だけをインデントしたい場合は `--no-html-indent` を指定します。

```bash
cargo run -- --no-html-indent samples/sample.html.erb
erbfmt --no-html-indent samples/sample.html.erb
```

同じ挙動は `erbfmt.json` の `formatter.noHtmlIndent` でも設定できます。

`formatter.lineWidth` は、長いHTMLタグを属性ごとに複数行へ展開する基準として使われます。

## VSCode

このリポジトリには、保存時に `.html.erb` ファイルを整形する workspace 設定が含まれています。
また、`*.html.erb` を `erb` language id に関連付ける設定も含まれています。これにより、Shopify Ruby LSP などのRuby toolingが `.html.erb` を認識しやすくなります。

推奨拡張の `emeraldwalk.RunOnSave` をインストールすると、`.html.erb` ファイル保存時に以下のコマンドが実行されます。

```bash
cargo run --quiet -- --write "${file}"
```

workspace の language association と将来の拡張方針は [docs/VSCode.md](docs/VSCode.md) にまとめています。

## 将来構想

- Lintルールの拡充
- npm package
- Ruby Gem

ローカルリリース確認手順は [docs/Release.md](docs/Release.md) にまとめています。
formatter / linter 設定は [docs/Configuration.md](docs/Configuration.md) にまとめています。
