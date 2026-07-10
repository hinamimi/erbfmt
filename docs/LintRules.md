# Lint Rule Design

この文書は、現在の `erbfmt --lint` の挙動と、次に実装するlint ruleの設計をまとめます。

## 方針

- Ruby AST parsing はしない。
- Rails semantic analysis はしない。
- まずは lexer / mixed parser が既に持っている構造だけを使う。
- diagnostic は CLI と VSCode の両方で読みやすい短いmessageにする。
- `info` / `warn` / `error` はCLIとVSCode diagnosticsのseverityに反映する。
- CLIは `error` diagnostic があれば失敗し、infoやwarningだけなら成功する。
- `erbfmt-ignore` comment directive で次行のlint診断を抑制できる。

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

### ignore directive

HTML comment または ERB comment で、次の物理行のlint診断を抑制できます。

```erb
<!-- erbfmt-ignore lint/noDeprecatedHtmlTag: legacy markup -->
<center>Legacy</center>
```

```erb
<%# erbfmt-ignore lint/emptyErbCodeTag: generated placeholder %>
<% %>
```

rule名を省略した `erbfmt-ignore lint` は、次行のlint診断をすべて抑制します。

詳しい構文は [Ignore.md](Ignore.md) を参照してください。

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

### `noDeprecatedHtmlAttribute`

HTML5ではobsolete、またはCSS/semantic markupへ移行した方がよい古いHTML attributeを
検出します。

対象例:

```erb
<a name="legacy"></a>
<table summary="Legacy table" align="center" cellpadding="0">
<td bgcolor="#fff" valign="middle">
<iframe frameborder="0"></iframe>
```

message:

```text
deprecated HTML attribute `summary` on <table>: use CSS or semantic markup instead
```

初期severity:

- `info`

理由:

- メールHTMLや古い埋め込みHTMLでは、`align`, `bgcolor`, `valign`, `frameborder`
  などが実用上まだ使われます。
- `unknown HTML attribute` として扱うと誤検知に見えやすい一方、完全に黙ると
  モダンなHTMLへ寄せたいユーザーが気づけません。
- デフォルトは `info` として通知だけ行い、project方針に応じて `off`, `warn`,
  `error` を選べるようにします。

config:

```json
{
  "linter": {
    "rules": {
      "noDeprecatedHtmlAttribute": "info"
    }
  }
}
```

### `noDuplicateHtmlAttribute`

同じHTML tag内の重複attributeを検出します。

対象例:

```erb
<article class="card" id="one" class="wide"></article>
<button data-action="save" data-action="submit"></button>
```

message:

```text
duplicate HTML attribute `class`
duplicate HTML attribute `data-action`
```

range:

- 重複している2つ目以降のattribute名の開始位置

方針:

- attribute名はASCII case-insensitiveに扱います。
- `class`, `id`, `data-*`, `aria-*` などを同じruleで検出します。
- tag内にERB fragmentを含む場合は、Ruby/ERB側でattributeが生成される可能性があるため、
  このruleでは診断しません。

config:

```json
{
  "linter": {
    "rules": {
      "noDuplicateHtmlAttribute": "error"
    }
  }
}
```

### `noInvalidHtmlBooleanAttribute`

HTML boolean attribute の危険または冗長な値を検出します。

対象例:

```erb
<button disabled="false">Save</button>
<input checked="checked">
```

message:

```text
invalid HTML boolean attribute value `disabled="false"`
redundant HTML boolean attribute value `checked="checked"`
```

range:

- 対象attribute名の開始位置

方針:

- `disabled`, `checked`, `selected`, `readonly`, `required` などのHTML boolean
  attributeを対象にします。
- `disabled="false"` は属性が存在する時点で有効扱いになるため、危険な値として検出します。
- `checked="checked"` のように属性名と同じ値は、HTMLとしては動作しますが冗長な値として検出します。
- 値なしの `disabled` や `checked` は許可します。
- tag内にERB fragmentを含む場合は、Ruby/ERB側で値が生成される可能性があるため、
  このruleでは診断しません。

config:

```json
{
  "linter": {
    "rules": {
      "noInvalidHtmlBooleanAttribute": "error"
    }
  }
}
```

### `noNonDoubleQuotedHtmlAttributeValue`

HTML attribute value がdouble quoteで囲まれていないケースを検出します。

対象例:

```erb
<div class=<%= foo %>></div>
<div class=card></div>
<div class='card'></div>
```

message:

```text
HTML attribute value must use double quotes `class=<%= foo %>`
HTML attribute value must use double quotes `class=card`
HTML attribute value must use double quotes `class='card'`
```

range:

- 対象attribute名の開始位置

方針:

- 値を持つHTML attributeはdouble quoteで囲むことを推奨します。
- 値なしのboolean attribute、たとえば `disabled` や `hidden` は許可します。
- ERB outputを値にする場合も、`class="<%= foo %>"` のようにHTML側でdouble
  quoteを明示することを推奨します。

config:

```json
{
  "linter": {
    "rules": {
      "noNonDoubleQuotedHtmlAttributeValue": "error"
    }
  }
}
```

### `noUnknownHtmlAttribute`

HTML tag に存在しない、または typo の可能性が高い attribute を検出します。

このruleは最初からHTML仕様全体を網羅しません。まず利用頻度が高く、誤検知時の影響を
制御しやすいtagから段階的に実装します。

初期実装対象tag:

- `input`
- `a`
- `button`
- `form`
- `label`
- `img`
- `script`
- `link`
- `meta`
- `textarea`
- `select`
- `option`
- `audio`
- `video`
- `source`
- `track`
- `iframe`
- `object`
- `embed`
- `map`
- `area`
- `table`
- `colgroup`
- `col`
- `th`
- `td`
- `ol`
- `ul`
- `li`
- `details`
- `dialog`
- `progress`
- `meter`
- `time`
- `canvas`
- `base`
- `style`

メールHTMLや埋め込みHTMLで実運用上よく使われる古い属性 (`align`, `bgcolor`,
`valign`, `frameborder` など) は、unknown attributeとしては扱いません。これらは
`noDeprecatedHtmlAttribute` で `info` として検出します。

今後追加するtag:

- `del`
- `ins`
- `blockquote`
- `q`
- `data`
- `output`

初期対象外:

- custom element (`my-component` のようにtag名に `-` を含むもの)
- SVG / MathML foreign content
- tag内にERB fragmentを含むattribute list
- `data-*`
- `aria-*`
- framework / library specific attribute

### severity

初期値は `warn` を想定します。

理由:

- Rails viewではStimulus、Vue、Alpine.js、Hotwire系のattributeが混ざることがある。
- HTML仕様に存在しないattributeが、projectやlibraryでは意味を持つ場合がある。
- 初期実装では対応tagとattribute setが限定的なため、errorにすると導入時の摩擦が大きい。

config:

```json
{
  "linter": {
    "rules": {
      "noUnknownHtmlAttribute": "warn"
    }
  }
}
```

### attribute分類

attributeは次の順で判定します。

1. HTML global attribute
2. `data-*`
3. `aria-*`
4. configured ignore pattern
5. framework / library known attribute
6. tag-specific attribute
7. typo candidate
8. unknown attribute

### global attribute

まずHTML global attributeを共有setとして持ちます。

初期set:

- `accesskey`
- `autocapitalize`
- `autofocus`
- `class`
- `contenteditable`
- `dir`
- `draggable`
- `enterkeyhint`
- `hidden`
- `id`
- `inert`
- `inputmode`
- `is`
- `itemid`
- `itemprop`
- `itemref`
- `itemscope`
- `itemtype`
- `lang`
- `nonce`
- `part`
- `popover`
- `slot`
- `spellcheck`
- `style`
- `tabindex`
- `title`
- `translate`

event handler attribute (`onclick` など) は、HTML仕様上は存在するattributeとして
初期実装では許可します。inline event handlerを避けたい場合は、security/style lintとして
別ruleに分けます。

### tag-specific attribute

tagごとのattribute setは、小さく始めて増やします。

`input` 初期set:

- `accept`
- `alt`
- `autocomplete`
- `capture`
- `checked`
- `dirname`
- `disabled`
- `form`
- `formaction`
- `formenctype`
- `formmethod`
- `formnovalidate`
- `formtarget`
- `height`
- `list`
- `max`
- `maxlength`
- `min`
- `minlength`
- `multiple`
- `name`
- `pattern`
- `placeholder`
- `readonly`
- `required`
- `size`
- `src`
- `step`
- `type`
- `value`
- `width`

`a` 初期set:

- `download`
- `href`
- `hreflang`
- `ping`
- `referrerpolicy`
- `rel`
- `target`
- `type`

`button` 初期set:

- `command`
- `commandfor`
- `disabled`
- `form`
- `formaction`
- `formenctype`
- `formmethod`
- `formnovalidate`
- `formtarget`
- `name`
- `popovertarget`
- `popovertargetaction`
- `type`
- `value`

追加済みtag:

- `form`
- `label`
- `img`
- `script`
- `link`
- `meta`
- `textarea`
- `select`
- `option`

### framework / library attribute

HTMLでは未定義でも、frameworkやlibraryが使うattributeがあります。これらは初期実装では
warning対象にしません。

常に許可するpattern:

- `data-*`
- `aria-*`

初期のknown framework pattern:

- `v-*` (`v-model`, `v-if`, `v-bind`, `v-on` など)
- `:*` (Vue shorthand)
- `@*` (Vue event shorthand)
- `x-*` (Alpine.js)
- `hx-*` (htmx)
- `ng-*` (AngularJS / Angular template style)
- `phx-*` (Phoenix LiveView)
- `turbo-*` は原則 `data-turbo-*` を推奨するが、実プロジェクト互換のため最初は許可候補に入れる

Stimulus / Turbo は基本的に `data-controller`, `data-action`, `data-*-target`,
`data-*-value` を使うため、`data-*` 許可で対応します。

Rails helperが生成するattributeや、project独自のlibrary attributeを扱うために、
将来的にconfigを追加します。

想定config:

```json
{
  "linter": {
    "html": {
      "allowUnknownAttributes": [
        "v-*",
        "x-*",
        "hx-*",
        "my-library-*"
      ]
    }
  }
}
```

ただし、初期実装では `linter.rules.noUnknownHtmlAttribute` のseverityだけを公開し、
allowlist configは必要になった時点で追加します。

### typo detection

未知attributeのうち、既知attributeに近いものはtypo候補としてmessageを変えます。

対象例:

```erb
<button tab-index="0">Open</button>
<button tabIndex="0">Open</button>
<input auto-complete="email">
```

message案:

```text
unknown HTML attribute `tab-index`; did you mean `tabindex`?
unknown HTML attribute `tabIndex`; did you mean `tabindex`?
unknown HTML attribute `auto-complete`; did you mean `autocomplete`?
```

方針:

- HTML attribute名はASCII case-insensitiveに扱います。
- diagnosticの表示では元のattribute名を保持します。
- `tabIndex` のようなcamelCaseは、小文字化した上で既知attributeと比較します。
- `tab-index` のようなkebab-case typoは、`-` を除去した文字列でも比較します。
- 編集候補を出す場合はlinterではなくassist actionに寄せます。

初期typo候補:

- `tab-index` / `tabIndex` -> `tabindex`
- `auto-complete` / `autoComplete` -> `autocomplete`
- `read-only` / `readOnly` -> `readonly`
- `max-length` / `maxLength` -> `maxlength`
- `min-length` / `minLength` -> `minlength`
- `form-action` / `formAction` -> `formaction`
- `form-method` / `formMethod` -> `formmethod`
- `form-target` / `formTarget` -> `formtarget`
- `ariaLabel` -> `aria-label`

`ariaLabel` はHTMLの `aria-*` としては無効なのでtypo候補にします。ただし React/JSX
では `aria-label` ではなく `aria-label` をHTML文字列として書くため、ERBでは
`aria-label` を推奨します。

### ERBとの相互作用

tag内にERB fragmentがある場合は、初期実装では診断しません。

対象外例:

```erb
<input <%= field_options %> name="email">
<a href="<%= path %>" <%= link_options %>>Link</a>
```

理由:

- ERB側でattribute名そのものが生成される可能性がある。
- Ruby/Rails semantic analysisなしでattribute listを確定できない。
- 誤検知よりも安全側を優先する。

attribute valueにERB outputが含まれるだけなら診断対象にできます。

対象例:

```erb
<input auto-complete="<%= autocomplete %>">
```

### custom elements

custom elementは未定義attributeを許可します。

対象外例:

```erb
<my-combobox selected-value="1" on-select="save"></my-combobox>
```

理由:

- custom elementはproject固有attributeを持つ。
- HTML標準のtag-specific attribute setで判定すると誤検知が多い。

ただし、custom element上でもglobal attribute typoだけ検出したい需要はありえます。
これは将来のoptionとして検討します。

### assistとの関係

`noUnknownHtmlAttribute` は最初はlinter ruleです。

typo候補のうち、置換が明らかなものは将来的にassist actionへ移せます。

想定action:

- `html.fixAttributeName`

例:

```erb
- <button tab-index="0">Open</button>
+ <button tabindex="0">Open</button>
```

初期実装ではassistを同時に実装せず、diagnostic messageで候補だけ示します。

### 実装順

1. global attribute setを実装する。
2. `input`, `a`, `button` のtag-specific attribute setを実装する。
3. `data-*`, `aria-*`, framework patternを許可する。
4. `tab-index` / `tabIndex` などのtypo候補messageを実装する。
5. `form`, `label`, `img`, `script`, `link`, `meta`, `textarea`, `select`, `option`
   を追加する。
6. 実プロジェクトauditで誤検知を確認し、allowlist configが必要か判断する。

完了条件:

- `input`, `a`, `button`, `form`, `label`, `img`, `script`, `link`, `meta`,
  `textarea`, `select`, `option` で明らかなtypoを検出できる。
- `data-*`, `aria-*`, `v-model`, `x-*`, `hx-*` などを誤検知しない。
- tag内ERB fragmentを含む不確実なtagでは診断しない。
- messageがVS Code diagnostics上で短く読める。

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

- HTML attribute parser boundary hardening
- より広いHTML content model validation
- lint severity plumbing

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
3. HTML boolean attribute validation
4. lint severity plumbing

autocorrectは後回しにします。
