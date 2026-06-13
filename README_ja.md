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
- Lexer雛形

## 将来構想

- Formatter
- Linter
- VSCode拡張
- npm package
- Ruby Gem
