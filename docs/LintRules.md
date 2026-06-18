# Lint Rule Design

この文書は、現在の `erbfmt --lint` の挙動と、次に実装するlint ruleの設計をまとめます。

## 方針

- Ruby AST parsing はしない。
- Rails semantic analysis はしない。
- まずは lexer / mixed parser が既に持っている構造だけを使う。
- diagnostic は CLI と VSCode の両方で読みやすい短いmessageにする。
- VSCode diagnostics は現時点ではすべて `Error` として表示される。
- `warn` / `error` のseverity差分はまだ出さず、どちらもrule enabledとして扱う。

## ruleの分類

lint rule は次の層に分けて管理します。

- HTML-only rule: HTML fragment を HTML token stream として見て判定する。Ruby code
  と ERB control-flow には踏み込まない。
- ERB-structure rule: ERB lexer と mixed parser の block / branch 構造を使って判定する。
- Ruby-text rule: ERB tag 内部の text を軽く見る。Ruby AST parsing はしない。
- Common diagnostics: HTML と ERB の入れ子のように mixed parser が検出する構文診断。
  通常の rule と違い、基本的には `off` にしない。

この分類により、純粋なHTML、Ruby code、ERB control-flow、HTML/ERB共通構造を
同じ `linter.rules` に公開しつつ、実装上の責務は分けます。

## 現在のrule

### `noSelfClosingHtmlTag`

HTML5では self-closing slash は HTML element を閉じません。
そのため、self-closing HTML tagを検出します。

対象:

```erb
<div />
<br />
```

message:

```text
self-closing HTML tag `<div />` is not valid HTML5
```

range:

- self-closing HTML tagの開始位置

理由:

- 純粋なHTML fragmentだけで検出できる。
- 非void elementは `<div></div>` のように明示的に閉じる方が安全。
- void elementは `<br>` のようにslashなしで書く方針に寄せる。

config:

```json
{
  "linter": {
    "rules": {
      "noSelfClosingHtmlTag": "error"
    }
  }
}
```

### `noDeprecatedHtmlTag`

HTML5で非推奨またはobsoleteなHTML tagを検出します。

対象例:

```erb
<center>Legacy</center>
<font color="red">Alert</font>
```

message:

```text
deprecated HTML tag `<center>`
```

range:

- deprecated HTML tagの開始位置

理由:

- 純粋なHTML fragmentだけで検出できる。
- ERBやRuby codeの意味解析なしで、古いmarkupを早めに見つけられる。

config:

```json
{
  "linter": {
    "rules": {
      "noDeprecatedHtmlTag": "error"
    }
  }
}
```

### `noInvalidHtmlNesting`

HTML content model に反する代表的な親子関係を検出します。

対象例:

```erb
<ul>
  <div>Bad</div>
</ul>

<table>
  <tr><div>Bad</div></tr>
</table>

<p>
  <div>Bad</div>
</p>
```

message:

```text
invalid HTML nesting: <ul> cannot have <div> as a direct child
invalid HTML nesting: <tr> cannot have <div> as a direct child
invalid HTML nesting: <p> cannot contain <div>
```

range:

- 問題のある子HTML tag、またはtextの開始位置

現在検出するもの:

- `ul`, `ol`, `menu` 直下の `li`, `script`, `template` 以外の要素または非空text
- `table` 直下の `caption`, `colgroup`, `thead`, `tbody`, `tfoot`, `tr`, `script`,
  `template` 以外の要素または非空text
- `thead`, `tbody`, `tfoot` 直下の `tr`, `script`, `template` 以外の要素または非空text
- `tr` 直下の `td`, `th`, `script`, `template` 以外の要素または非空text
- `colgroup` 直下の `col`, `template` 以外の要素または非空text
- `p` 内の代表的な non-phrasing HTML tag

ERB block はHTMLの親子関係を壊さないtransparentな構造として扱います。
たとえば次は許可します。

```erb
<ul>
  <% items.each do |item| %>
    <li><%= item.name %></li>
  <% end %>
</ul>
```

理由:

- Rails / ERB templateで起きやすいHTML構造の崩れを早く見つけられる。
- Ruby AST parsing や Rails semantic analysis なしで検出できる。
- HTML parserの全面実装に入る前の、保守的なcontent model lintとして扱える。

config:

```json
{
  "linter": {
    "rules": {
      "noInvalidHtmlNesting": "error"
    }
  }
}
```

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
unsupported ERB block starter `while`
```

range:

- unsupported block開始tagの開始位置

現在supportするERB block starter:

- `if`
- `unless`
- `case`
- `do`
- `begin`
- output ERB do-block, for example `<%= form_with ... do |form| %>`

現在このruleで検出する未対応starter:

- `while`
- `for`
- `until`

理由:

- Ruby AST parsing なしで安全に検出できる。
- 未対応blockをformatterが誤って整形するより、lintで明示した方が安全。

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

### `emptyErbBranch`

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

## 次に実装するrule

候補:

- HTML attribute duplicate detection
- boolean attribute normalization warning
- より広いHTML content model validation

いずれも HTML-only rule として実装し、Ruby codeやERB control-flowには踏み込みません。

## ruleではなくdiagnostic品質改善として扱うもの

### HTML nesting diagnostics

現在の mixed parser はHTML tagの不整合を検出します。

例:

```erb
<div><span>Hello</div>
```

確認用fixture:

```bash
cargo run --quiet -- --lint samples/html-parse-errors.html.erb
```

現在のmessage:

```text
mismatched HTML close tag `</div>`, expected `</span>`
```

location:

- `unexpected close`: unexpected close tag 側
- `mismatched close`: mismatched close tag 側
- `unclosed open`: unclosed open tag 側

これは構文診断であり、通常のlint ruleとして `off` にできるものではありません。

### `unsupportedErbBlockStarter` message refinement

既存ruleの改善として扱います。

今後の改善:

- support済みblock starter一覧をmessageに含めすぎない。
- docs側でMVP supported starterを示す。
- rangeは今のtag開始位置を維持する。

## 実装順

1. `unsupportedErbBlockStarter` message refinement
2. HTML attribute duplicate detection
3. boolean attribute normalization warning

severity plumbingやautocorrectは後回しにします。
