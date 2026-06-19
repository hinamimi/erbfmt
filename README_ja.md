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
- syntax / lint finding の line / column 診断
- `erbfmt.json` による formatter / linter 設定
- `formatter.lineWidth` による長いHTMLタグと単独ERB tagの折りたたみ
- 複数ファイルのlint、check、write
- formatter、diagnostics、syntax highlighting、ERB向けコメントtoggleを持つ薄いVSCode extension

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
カレントディレクトリに設定ファイルを作る場合は `init` を使います。

```bash
erbfmt init
erbfmt init --force
```

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

`--write`、`--check`、`--lint` は同時に指定できません。

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

HTMLタグ階層によるインデントを無効にし、ERB制御構文だけをインデントしたい場合は `erbfmt.json` の `"indentHtml": false` を設定します。

`formatter.lineWidth` は、長いHTMLタグを属性ごとの複数行へ展開し、閉じマーカーを独立行にする基準として使われます。

長い単独のERB tagにも `formatter.lineWidth` を使いますが、erbfmtはRuby式そのものは分割しません。
長すぎる場合はERB tagの外枠だけを展開します。

```erb
<%=
  link_to "Edit profile", edit_user_path(user), class: "button button--primary"
%>
```

## サンプル

- `samples/sample.html.erb`: 意図的に未整形のformatterデモ用サンプル
- `samples/stability.html.erb`: formatter出力の安定性を見る固定fixture
- `samples/formatter-audit.html.erb`: Railsらしいtemplateを使ったformatter監査fixture
- `samples/formatter-edge-cases.html.erb`: formatter edge case fixture
- `samples/real-template-audit.html.erb`: table、turbo-frame、renderを含む実テンプレート寄り監査fixture
- `samples/lint-next.html.erb`: 意図的にlint issueを含むfixture
- `samples/html-parse-errors.html.erb`: 意図的にHTML閉じタグ不一致を含むfixture

## VSCode

このリポジトリには、薄いVSCode extension scaffold が `editors/vscode` に含まれています。
`*.html.erb` ファイルの document formatter として `erbfmt` を登録しつつ、formatter / lint engine はRust binaryに残します。extension は open/save 時に `erbfmt --lint` も呼び出し、diagnostics を表示します。

ローカル開発では、先にbinaryをbuildします。

```bash
cargo build
```

このcheckoutから実行している場合、extension は `target/debug/erbfmt` があればそれを使います。
別のbinaryを使う場合は、wrapper が呼び出すcommandを明示できます。

```json
{
  "erbfmt.command": "/absolute/path/to/erbfmt",
  "erbfmt.arguments": []
}
```

workspace には fallback として RunOnSave 設定も含まれています。

`.html.erb` ファイル保存時に以下のコマンドが実行されます。

```bash
cargo run --quiet -- --write "${file}"
```

VSCode extension と workspace integration の詳細は [docs/VSCode.md](docs/VSCode.md) にまとめています。

## 将来構想

- Lintルールの拡充
- npm package
- Ruby Gem

ローカルリリース確認手順は [docs/Release.md](docs/Release.md) にまとめています。
formatter / linter 設定は [docs/Configuration.md](docs/Configuration.md) にまとめています。
現在と今後のlint rule設計は [docs/LintRules.md](docs/LintRules.md) にまとめています。
最初の公開release計画は [docs/FirstRelease.md](docs/FirstRelease.md) にまとめています。
VSCode extension のpackageとlocal installは [docs/VSCode.md](docs/VSCode.md) にまとめています。
バイナリ配布方針は [docs/Distribution.md](docs/Distribution.md) にまとめています。
