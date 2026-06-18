# Lint Rule Design

この文書は、現在の `erbfmt --lint` の挙動と、次に実装するlint ruleの設計をまとめます。

## 方針

- Ruby AST parsing はしない。
- Rails semantic analysis はしない。
- まずは lexer / mixed parser が既に持っている構造だけを使う。
- diagnostic は CLI と VSCode の両方で読みやすい短いmessageにする。
- VSCode diagnostics は現時点ではすべて `Error` として表示される。
- `warn` / `error` のseverity差分はまだ出さず、どちらもrule enabledとして扱う。

## 現在のrule

### `emptyErbControlBlock`

空のERB control blockを検出します。

対象:

```erb
<% if show_empty_state %>
<% end %>
```

message:

```text
empty ERB control block `<% if show_empty_state %>`
```

range:

- block開始tagの開始位置

### `unsupportedErbBlockStarter`

MVPでまだsupportしないERB block starterを検出します。

対象:

```erb
<% while job.running? %>
  <p>Waiting</p>
```

message:

```text
unsupported ERB block starter `<% while job.running? %>`
```

range:

- unsupported block開始tagの開始位置

### `emptyErbCodeTag`

空のERB tagを検出します。

対象:

```erb
<% %>
<%= %>
```

message:

```text
empty ERB code tag `<% %>`
empty ERB output tag `<%= %>`
```

range:

- 空のERB tagの開始位置

理由:

- lexer tokenだけで検出できる。
- Ruby AST parsing が不要。
- 空のtagはformatterで自然に直すより、lintで明示した方が安全。

config:

```json
{
  "linter": {
    "rules": {
      "emptyErbCodeTag": "error"
    }
  }
}
```

## 次に実装するrule

### 1. `emptyErbBranch`

空のERB branchを検出します。

対象:

```erb
<% if current_user %>
  <p>Hello</p>
<% else %>
<% end %>
```

```erb
<% case role %>
<% when "admin" %>
<% when "member" %>
  <p>Member</p>
<% end %>
```

message:

```text
empty ERB branch `<% else %>`
empty ERB branch `<% when "admin" %>`
```

range:

- 空のbranch tagの開始位置

理由:

- 現在の `emptyErbControlBlock` では、block全体に内容があれば空branchを検出しない。
- branchごとにmeaningful contentを追跡すれば、token列だけで検出できる。
- VSCode上でbranch tagにdiagnosticを置ける。

config:

```json
{
  "linter": {
    "rules": {
      "emptyErbBranch": "error"
    }
  }
}
```

## ruleではなくdiagnostic品質改善として扱うもの

### HTML nesting diagnostics

現在の mixed parser はHTML tagの不整合を検出します。

例:

```erb
<div><span>Hello</div>
```

現状のmessage:

```text
mismatched HTML close tag `</div>`, expected closing tag for `span`, found `div`
```

今後の改善:

- close tag側のline / columnをより安定して出す。
- messageをVSCode上で読んだときに短くする。
- `unexpected close`, `mismatched close`, `unclosed open` のmessageを揃える。

これは構文診断であり、通常のlint ruleとして `off` にできるものではありません。

### `unsupportedErbBlockStarter` message refinement

既存ruleの改善として扱います。

今後の改善:

- support済みblock starter一覧をmessageに含めすぎない。
- docs側でMVP supported starterを示す。
- rangeは今のtag開始位置を維持する。

## 実装順

1. `emptyErbBranch`
2. HTML nesting diagnostics のmessage / location改善

`emptyErbBranch` を先に実装し、severity plumbingやautocorrectは後回しにします。
