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
- AST Parser
- 基本的なFormatter
- ERB制御構文のインデント
- HTMLタグ階層のインデント
- `--write` によるファイルの直接整形
- VSCode workspace の保存時整形設定
- Lexer / Parser 診断を使った基本的なLinter
- `--check` による整形済みチェック

## CLI

ファイルを整形します。

```bash
cargo run -- samples/sample.html.erb
```

ファイルへ直接書き戻す場合は `--write` を指定します。

```bash
cargo run -- --write samples/sample.html.erb
```

ファイルをlintする場合は `--lint` を指定します。

```bash
cargo run -- --lint samples/sample.html.erb
```

ファイルが整形済みかどうかだけを確認する場合は `--check` を指定します。

```bash
cargo run -- --check samples/sample.html.erb
```

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
  <% end %>
</div>
```

HTMLタグ階層によるインデントを無効にし、ERB制御構文だけをインデントしたい場合は `--no-html-indent` を指定します。

```bash
cargo run -- --no-html-indent samples/sample.html.erb
```

## VSCode

このリポジトリには、保存時に `.html.erb` ファイルを整形する workspace 設定が含まれています。
推奨拡張の `emeraldwalk.RunOnSave` をインストールすると、`.html.erb` ファイル保存時に以下のコマンドが実行されます。

```bash
cargo run --quiet -- --write "${file}"
```

## 将来構想

- Lintルールの拡充
- npm package
- Ruby Gem
