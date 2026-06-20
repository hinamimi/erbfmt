# Roadmap

## 方針

erbfmt は `*.html.erb` 向けの formatter / linter です。
目標は Prettier、Biome、dprint に近い体験を ERB template に提供することです。

中核の formatter / linter engine は Rust に残します。VSCode extension、npm
package、Ruby gem は将来的にも薄い wrapper として扱い、別実装の formatter
engine は持ちません。

Ruby code は現時点では Ruby AST として解析しません。ERB tag 内部の式は text
として保持し、HTML と ERB control-flow marker の構造を使って整形します。

## 現在のベースライン

実装済み:

- Rust CLI
- single-file / multi-file formatting
- `--write`, `--check`, `--lint`
- ERB lexer
- lightweight HTML tokenizer
- HTML-aware mixed parser
- mixed AST-driven formatter
- `if`, `unless`, `case`, `do`, `begin`, `end` の ERB block formatting
- `else`, `elsif`, `when`, `rescue`, `ensure` の ERB branch formatting
- `<%= form_with ... do |form| %>` のような output ERB do-block formatting
- HTML tag 内の ERB output attribute handling
- `formatter.lineWidth` による長い HTML tag と standalone ERB tag の wrapping
- long standalone ERB tag の safe wrapping
- basic syntax lint rules
- empty ERB code tag / branch lint rules
- HTML5 self-closing / deprecated tag lint rules
- HTML duplicate / boolean attribute lint rules
- HTML content model lint rules for list / table / paragraph nesting
- lexer / parser / linter diagnostics の line / column reporting
- `erbfmt.json` による formatter / linter config
- `insta` snapshot tests
- formatter idempotency tests
- CLI integration tests
- release checklist
- local install docs
- binary distribution strategy docs
- manual GitHub Actions workflow for binary archive artifacts
- VSCode `html-erb` language id
- VSCode TextMate syntax highlighting
- VSCode document formatter wrapper
- VSCode diagnostics wrapper for `erbfmt --lint`
- VSCode ERB-safe `Ctrl+/` comment toggling
- VSCode local VSIX packaging
- platform-specific Ruby gem wrapper
- 4platformのbinary / gem release rehearsal

参照する主な文書:

- [Configuration.md](Configuration.md): `erbfmt.json` の仕様
- [Release.md](Release.md): release verification と versioning
- [Distribution.md](Distribution.md): binary distribution strategy
- [RubyGem.md](RubyGem.md): platform-specific Ruby gem wrapper design
- [VSCode.md](VSCode.md): VSCode extension と local package

参照する主な sample:

- `samples/sample.html.erb`: 意図的に未整形の formatter demo
- `samples/lint-next.html.erb`: lint rule sample
- `samples/stability.html.erb`: formatting stability sample
- `samples/formatter-audit.html.erb`: real-template formatter audit sample
- `samples/formatter-edge-cases.html.erb`: focused formatter edge-case sample
- `samples/real-template-audit.html.erb`: table / turbo-frame / render-heavy audit sample
- `samples/html-parse-errors.html.erb`: HTML close tag mismatch diagnostic sample

既知の制約:

- Ruby AST parsing はまだしない。
- npm package はまだない。
- Ruby gem は実装済みだがRubyGems.orgへはまだ公開していない。
- VSCode extension は実装済みだがまだ公開していない。
- VSCode extension は Rust binary をまだ bundle / download しない。
- GitHub Release はまだ publish しない。

## 次に重視すること

当面は「GitHub に置いたときに早期利用者が試せる状態」を優先します。

優先順位:

1. GitHub 上で CI / docs / packaging が問題なく見えることを確認する。
2. prebuilt binary の作成結果を確認し、artifact 名と checksum を安定させる。
3. formatter の実テンプレート耐性を上げる。
4. VSCode extension が binary を見つける体験を改善する。
5. npm / Ruby gem などの wrapper は、release binary が安定してから設計する。

## Milestone 36

初回 GitHub push verification

Status: Done

GitHub に公開した直後の repository 状態を確認します。

やること:

- `main` branch の GitHub Actions が動くか確認する。
- CI failure があれば原因を切り分けて修正する。
- README、LICENSE、workflow、docs が GitHub 上で読みやすく表示されるか確認する。
- VSCode package metadata が GitHub 上の repository と矛盾していないか確認する。

完了条件:

- GitHub CI の状態が把握できている。
- 初回 push 固有の問題があれば修正済み、または明示的に次へ送られている。
- repository landing page が早期 contributor に読める状態になっている。

範囲外:

- package publishing
- GitHub Release publishing
- npm / Ruby gem implementation

結果:

- `gh run list --repo hinamimi/erbfmt --branch main --limit 10` で `main`
  branch の CI が成功していることを確認した。
- 最新 CI run は Rust job と VSCode Extension job の両方が成功していた。
- GitHub repository は `hinamimi/erbfmt`、default branch は `main` と確認した。
- CI annotation で出ていた Node.js 20 deprecation に対応するため、
  `actions/checkout` と `actions/setup-node` を v5 に更新した。
- package publishing と GitHub Release publishing は予定どおり範囲外のままにした。

## Milestone 37

Release binary artifact verification

Status: Done

manual `Release Binaries` workflow と `scripts/package-binary.sh` の出力を確認し、
prebuilt binary 配布の土台を固めます。

やること:

- GitHub Actions の binary artifact が想定 platform matrix で作られるか確認する。
- archive 名と `.sha256` 名が [Distribution.md](Distribution.md) と一致するか確認する。
- Linux / macOS / Windows の workflow failure を必要に応じて修正する。
- local `scripts/package-binary.sh` と workflow artifact の naming rule を揃える。

完了条件:

- 少なくとも workflow artifact の成功 / 失敗理由が明確になっている。
- artifact naming と checksum policy が docs と一致している。
- public release 前に残る課題が短く整理されている。

範囲外:

- GitHub Release の自動作成
- version bump
- binary download logic

結果:

- `Release Binaries` workflow を手動実行し、Linux x64、macOS arm64、
  Windows x64 の artifact が作成されることを確認した。
- 成功した artifact 名は [Distribution.md](Distribution.md) の命名規則と一致していた。
- Linux x64 と Windows x64 の `.sha256` は検証に成功した。
- archive の中身は binary、`LICENSE.txt`、`README.md` で揃っていた。
- macOS arm64 は package step で `sha256sum` がなく失敗したため、
  workflow に `shasum -a 256` fallback を追加した。
- `actions/upload-artifact` の Node.js 20 deprecation annotation に対応するため v6 に更新した。
- macOS x64 は古い `macos-13` runner label で待機し続けたため、
  GitHub-hosted runner の現行 label に合わせて `macos-15-intel` へ更新した。
- `scripts/package-binary.sh` は現hostで成功した。
- 修正後の workflow は次回 push 後に再度 `workflow_dispatch` で確認する。

## Milestone 38

Formatter real-template audit pass

Status: Done

実際の Rails / ERB template で不自然になりやすい formatter behavior を追加で確認します。

やること:

- `section`, `form`, `table`, `turbo-frame`, `render` などを含む実用寄り fixture を増やす。
- long HTML attribute wrapping の snapshot を追加する。
- ERB output と text が混ざる inline context の snapshot を追加する。
- unsupported pattern は無理に整形せず、制約として記録する。

完了条件:

- 追加した behavior が snapshot test で固定されている。
- idempotency test の対象を必要に応じて増やしている。
- 既存 sample の意図的な未整形状態を壊していない。

範囲外:

- Ruby AST parsing
- Rails semantic analysis
- formatter の全面 rewrite

結果:

- `samples/real-template-audit.html.erb` を追加した。
- `section`, `turbo-frame`, `table`, `render`, long attributes、inline ERB output を含む
  Rails寄りのfixtureとして現在のformatter behaviorを確認した。
- long HTML attributes は属性ごとの複数行に展開されることをsnapshotで固定した。
- standalone ERB output はERB markerだけを展開し、Ruby式そのものは分割しない方針を維持した。
- 新fixtureのsnapshot testとidempotency testを追加した。
- 新fixtureはlint issueなしで通ることを確認した。

## Milestone 39

Lint rule design pass

Status: Done

現在の parser / diagnostics を前提に、次に増やす lint rule を設計します。

候補:

- unmatched HTML tag diagnostics の改善
- unsupported ERB block starter の message 改善
- empty ERB control block の quick fix 方針
- ERB block と HTML nesting の mismatch warning
- rule severity と `erbfmt.json` schema の整理

完了条件:

- 次に実装する lint rule が 1 つから 3 つに絞られている。
- rule ごとの diagnostics range と message 方針が決まっている。
- VSCode diagnostics で見たときの体験が想定されている。

範囲外:

- Ruby semantic lint
- autocorrect
- LSP implementation

結果:

- [LintRules.md](LintRules.md) を追加した。
- 次に実装するlint ruleを `emptyErbCodeTag` と `emptyErbBranch` に絞った。
- `emptyErbCodeTag` は空の `<% %>` / `<%= %>` をtag開始位置で診断する。
- `emptyErbBranch` は空の `else`, `elsif`, `when`, `rescue`, `ensure` branchを
  branch tag開始位置で診断する。
- HTML nesting diagnostics は通常のlint ruleではなく、parser diagnostic品質改善として扱う。
- `warn` / `error` のseverity差分とautocorrectは後回しにした。

## Milestone 40

VSCode binary resolution UX

Status: Done

VSCode extension が `erbfmt` binary を見つけられないときの体験を改善します。

やること:

- current checkout の `target/debug/erbfmt` detection を確認する。
- `erbfmt.command` と `erbfmt.arguments` の説明を整理する。
- `erbfmt: Show Command` の出力を必要に応じて改善する。
- prebuilt binary が存在する前提で、download / cache 方式の設計だけ行う。

完了条件:

- local development と installed binary のどちらでも説明が一貫している。
- `ENOENT` / `EACCES` 時に次に何をすればよいか分かる。
- binary bundling / download の実装判断ができる状態になっている。

範囲外:

- 実際の binary download implementation
- VSCode Marketplace publishing

結果:

- VSCode extension の command resolution に source 情報を追加した。
- `erbfmt: Show Command` で resolution source、command line、cwd、checkout root、
  checkout binary、config path、setup hint を確認できるようにした。
- command解決順序を docs / extension README に明記した。
- `erbfmt.command` と `erbfmt.arguments` の設定説明を整理した。
- 将来のbinary download/cache方針を docs に追加した。
- 実際のdownload実装とMarketplace publishingは範囲外のままにした。

## Milestone 41

First public release planning

Status: Done

`0.0.0-dev` から最初の公開 release に進むための作業を設計します。

やること:

- 最初の version number を決める。
- release branch / tag / GitHub Release の手順を決める。
- public binary release 前の必須 verification を確定する。
- VSCode extension、npm package、Ruby gem をいつ出すか再判断する。

完了条件:

- release checklist が実行可能な手順になっている。
- version bump 対象ファイルが明確になっている。
- 初回 release で公開するものと公開しないものが明確になっている。

範囲外:

- 実際の public release
- npm / Ruby gem publishing

結果:

- [FirstRelease.md](FirstRelease.md) を追加した。
- 最初の公開version候補を `0.1.0` に決めた。
- 初回公開物は GitHub Release のRust CLI binary archivesと `.sha256` に絞った。
- crates.io、npm package、Ruby gem、VSCode Marketplace publishing は初回release範囲外にした。
- version bump対象ファイルを明確にした。
- release commit、annotated tag、manual `Release Binaries` workflow、draft GitHub Release の手順を整理した。
- 初回release後に `main` を `0.0.0-dev` へ戻すかは別milestoneで判断する。

## Milestone 42

`emptyErbCodeTag` lint rule

Status: Done

空の ERB code tag を lint で検出します。

対象:

- `<% %>`
- `<%= %>`
- whitespace だけを含む `<%   %>` / `<%=   %>`

やること:

- `erbfmt.json` の linter rules に `emptyErbCodeTag` を追加する。
- schema、configuration docs、lint rule docs を更新する。
- parser / lexer の既存 token 情報を使い、tag 開始位置に diagnostic を出す。
- CLI lint と VSCode diagnostics の両方で自然に見える message にする。
- unit test と CLI test を追加する。

完了条件:

- default で空の ERB code tag を検出できる。
- config で rule を無効化できる。
- `samples/lint-next.html.erb` で改善前後の見え方を確認できる。
- formatter behavior には影響しない。

範囲外:

- autocorrect
- Ruby AST parsing
- rule severity の `warn` / `error` 分岐

結果:

- `emptyErbCodeTag` を default enabled の lint rule として実装した。
- 空の `<% %>` と `<%= %>` を tag 開始位置で診断するようにした。
- 空白だけの ERB code / output tag も空 tag として扱う。
- 空 tag は meaningful content として数えず、空 control block の検出を邪魔しないようにした。
- `erbfmt.json`、schema、configuration docs、lint rule docs を更新した。
- `samples/lint-next.html.erb` に空 tag の例を追加した。
- unit test と CLI integration test を追加した。

## Milestone 43

`emptyErbBranch` lint rule

Status: Done

空の ERB branch を lint で検出します。

対象:

- `else`
- `elsif`
- `when`
- `rescue`
- `ensure`

やること:

- branch tag から次の branch / end までの meaningful content 判定を整理する。
- HTML whitespace と ERB whitespace の扱いを決める。
- `erbfmt.json` の linter rules に `emptyErbBranch` を追加する。
- rule docs、schema、test fixture を更新する。

完了条件:

- 空 branch を安定して検出できる。
- non-empty branch を誤検出しない snapshot / unit test がある。
- VSCode diagnostics で branch tag 開始位置に自然に表示される。

範囲外:

- Rails helper の意味解析
- autocorrect

結果:

- `emptyErbBranch` を default enabled の lint rule として実装した。
- 空の `else`, `elsif`, `when`, `rescue`, `ensure` branch を branch tag 開始位置で診断するようにした。
- branch 内の HTML whitespace、HTML comment、空 ERB code / output tag は meaningful content として数えないようにした。
- non-empty branch を誤検出しない unit test を追加した。
- config で `emptyErbBranch` を無効化できるようにした。
- `erbfmt.json`、schema、configuration docs、lint rule docs を更新した。
- `samples/lint-next.html.erb` に空 branch の例を追加した。
- unit test と CLI integration test を追加した。

## Milestone 44

HTML lint architecture and HTML5 rule pass

Status: Done

純粋なHTML、Ruby code、ERB control-flow、HTML/ERB共通構造を管理しやすい形に
分けながら、HTML-only lint rule を追加します。

やること:

- lint rule を HTML-only / ERB-structure / Ruby-text / common diagnostics に分類する。
- HTML token に相対位置情報を持たせ、HTML tag単位でdiagnosticを出せるようにする。
- HTML5でself-closing slashが問題になるtagを検出する。
- HTML5で非推奨またはobsoleteなtagを検出する。
- `erbfmt.json` の linter rules、schema、docs、sampleを更新する。

完了条件:

- `noSelfClosingHtmlTag` をdefaultで有効化し、configで無効化できる。
- `noDeprecatedHtmlTag` をdefaultで有効化し、configで無効化できる。
- HTML fragment内のtag開始位置にdiagnosticを出せる。
- `samples/lint-next.html.erb` でHTML-only ruleの見え方を確認できる。
- 既存 formatter behavior に影響しない。

範囲外:

- Ruby AST parsing
- autocorrect
- HTML parser の全面rewrite

結果:

- [LintRules.md](LintRules.md) に rule分類方針を追加した。
- HTML-only rule、ERB-structure rule、Ruby-text rule、common diagnostics を分けて管理する方針にした。
- HTML tokenizerに相対spanを追加し、HTML fragment内のtag開始位置を元ファイルのline/columnへ戻せるようにした。
- `noSelfClosingHtmlTag` を追加し、`<div />` や `<br />` のようなself-closing HTML tagを検出するようにした。
- `noDeprecatedHtmlTag` を追加し、`center`, `font`, `marquee` などの古いHTML tagを検出するようにした。
- `erbfmt.json`、schema、configuration docs、lint rule docsを更新した。
- `samples/lint-next.html.erb` にHTML-only lint例を追加した。
- unit testとCLI integration testを追加した。

## Milestone 45

HTML content model lint pass

Status: Done

`ul` / `ol` / `table` / `p` など、HTML content model の代表的な違反を検出します。

やること:

- `ul`, `ol`, `menu` 直下の `li` 以外の代表的な違反を検出する。
- `table`, `thead`, `tbody`, `tfoot`, `tr`, `colgroup` の代表的な構成違反を検出する。
- `p` 内の代表的な non-phrasing HTML tag を検出する。
- ERB block をHTML親子関係上transparentに扱う。
- `erbfmt.json` の linter rules、schema、docs、sampleを更新する。

完了条件:

- `noInvalidHtmlNesting` をdefaultで有効化し、configで無効化できる。
- list / table / p の主要ケースで問題のある子tagの開始位置にdiagnosticを出せる。
- ERB block を挟んだvalidなlist/table構造を誤検出しない。
- 既存 formatter behavior に影響しない。

範囲外:

- HTML content model の完全実装
- Ruby AST parsing
- autocorrect

結果:

- `noInvalidHtmlNesting` を追加した。
- `ul`, `ol`, `menu` 直下の `li`, `script`, `template` 以外の要素または非空textを検出するようにした。
- `table`, `thead`, `tbody`, `tfoot`, `tr`, `colgroup` の代表的な構成違反を検出するようにした。
- `p` 内の代表的な non-phrasing HTML tag を検出するようにした。
- ERB block をHTML親子関係上transparentに扱い、`ul > ERB block > li` のような構造を許可した。
- `erbfmt.json`、schema、configuration docs、lint rule docsを更新した。
- `samples/lint-next.html.erb` にlist / table / p のHTML nesting違反例を追加した。
- unit testとCLI integration testを追加した。

## Milestone 46

HTML diagnostics location / message refinement

Status: Done

HTML nesting diagnostics を VSCode diagnostics と CLI の両方で読みやすくします。

やること:

- unexpected close / mismatched close / unclosed open の message を整理する。
- close tag 側、open tag 側のどちらに diagnostic を置くかを case ごとに決める。
- mixed parser の HTML parse error が line / column をより安定して返せるか確認する。
- unit test と CLI integration test を追加する。
- [LintRules.md](LintRules.md) の diagnostic品質改善セクションを更新する。

完了条件:

- HTML tag 不整合の主要ケースで line / column が出る。
- VSCode上で見たときに短く理解できるmessageになっている。
- 既存 formatter behavior に影響しない。

範囲外:

- HTML parser の全面rewrite
- autocorrect
- HTML linter rule化

結果:

- `parse_spanned` が HTML fragment 内の tag span を使ってHTML parse errorのline/columnを出せるようにした。
- unexpected close と mismatched close は問題のあるclose tag側にdiagnosticを置く方針にした。
- unclosed open は閉じられなかったopen tag側にdiagnosticを置く方針にした。
- mismatched close のmessageを `expected closing tag for ... found ...` から `expected </...>` に短縮した。
- HTML parse error のCLI integration testを追加した。
- [LintRules.md](LintRules.md) のHTML diagnostics説明を更新した。

## Milestone 47

`unsupportedErbBlockStarter` message refinement

Status: Done

MVPでまだsupportしないERB block starterのmessageを、VSCode上で読みやすくします。

やること:

- `while`, `for`, `until` のunsupported messageを短く整理する。
- docs側にsupport済みblock starter一覧を置く。
- rule docsとCLI testを更新する。

完了条件:

- unsupported block starterのdiagnosticが短く分かりやすい。
- support済み / 未対応の境界がdocsで確認できる。
- 既存 formatter behavior に影響しない。

範囲外:

- Ruby AST parsing
- `while`, `for`, `until` のformatter support
- autocorrect

結果:

- unsupported ERB block starter のmessageを短縮した。
- `<% while job.running? %>` は keyword `while` を含む短いmessageで診断するようにした。
- `while`, `for`, `until` の未対応starterをunit testで固定した。
- [LintRules.md](LintRules.md) にsupport済みERB block starterと未対応starterを整理した。
- 既存 formatter behavior には影響しない。

## Milestone 48

HTML duplicate attribute lint rule

Status: Done

HTML tag内の重複attributeを検出します。

やること:

- HTML tagのraw属性部分からattribute名を軽く抽出する。
- `class`, `id`, `data-*`, `aria-*` などの重複を同じruleで検出する。
- ERB output attributeを含むtagで誤検出しない境界を決める。
- `erbfmt.json` の linter rules、schema、docs、sampleを更新する。
- unit testとCLI integration testを追加する。

完了条件:

- 同一HTML tag内の明らかな重複attributeを検出できる。
- ERB attribute fragmentを含むtagで危険な推測をしない。
- configでruleを無効化できる。

範囲外:

- 完全なHTML attribute parser
- Rails helper / Ruby semantic analysis
- autocorrect

結果:

- `noDuplicateHtmlAttribute` を追加した。
- HTML tag内の静的に読める重複attributeを検出するようにした。
- attribute名はASCII case-insensitiveに扱う方針にした。
- tag内にERB fragmentを含む場合は危険な推測を避け、このruleでは診断しない方針にした。
- `erbfmt.json`、schema、configuration docs、lint rule docsを更新した。
- `samples/lint-next.html.erb` に重複attribute例を追加した。
- unit testとCLI integration testを追加した。

## Milestone 49

HTML boolean attribute lint rule

Status: Done

HTML boolean attribute の冗長または不自然な書き方を検出します。

やること:

- `disabled="disabled"` や `checked="checked"` のような冗長なboolean attributeを検出する。
- `disabled="false"` のようにHTML上はtruthyになる危険な値を検出するか設計する。
- ERB valueを含むattributeで誤検出しない境界を決める。
- `erbfmt.json` の linter rules、schema、docs、sampleを更新する。
- unit testとCLI integration testを追加する。

完了条件:

- 明らかなboolean attribute issueを検出できる。
- ERB attribute valueを含むtagで危険な推測をしない。
- configでruleを無効化できる。

範囲外:

- autocorrect
- 完全なHTML attribute parser
- Rails helper / Ruby semantic analysis

結果:

- `noInvalidHtmlBooleanAttribute` を追加した。
- `disabled="false"` のようにHTML上はtruthyになる危険な値を検出するようにした。
- `checked="checked"` のようにattribute名と同じ値を持つ冗長なboolean attributeを検出するようにした。
- 値なしの `disabled` / `checked` などは許可する方針にした。
- tag内にERB fragmentを含む場合は危険な推測を避け、このruleでは診断しない方針にした。
- `erbfmt.json`、schema、configuration docs、lint rule docsを更新した。
- `samples/lint-next.html.erb` にboolean attribute例を追加した。
- unit testとCLI integration testを追加した。

## Milestone 50

Lint ignore directives

Status: Done

Biome / ESLint のように、局所的にlint診断を抑制するcomment directiveを追加します。

やること:

- HTML comment と ERB comment のignore構文を決める。
- 次の物理行のlint診断を抑制できるようにする。
- rule名を指定した場合は、そのruleだけを抑制できるようにする。
- formatter ignore は別Milestoneに分ける。
- docsとtestsを更新する。

完了条件:

- `<!-- erbfmt-ignore lint: reason -->` が次行のlint診断を抑制する。
- `<%# erbfmt-ignore lint/ruleName: reason %>` がERB commentとして使える。
- rule指定がある場合、他のruleのdiagnosticは残る。
- CLI integration testで挙動が固定されている。

範囲外:

- formatter ignore
- block-level disable / enable
- unused ignore directive reporting

結果:

- `erbfmt-ignore` と `erbfmt-ignore-next-line` を追加した。
- HTML comment と ERB comment の両方でlint ignore directiveを使えるようにした。
- `lint/ruleName` 形式で特定ruleだけを抑制できるようにした。
- [Ignore.md](Ignore.md) を追加した。
- [LintRules.md](LintRules.md) と [Configuration.md](Configuration.md) からignore仕様へリンクした。
- unit testとCLI integration testを追加した。

## Milestone 51

Lint severity plumbing

Status: Done

`erbfmt.json` の `warn` / `error` を、内部diagnosticとCLI / VSCode表示へ反映する土台を作ります。

やること:

- `Diagnostic` にseverityを持たせる設計を決める。
- `RuleSetting::Warn` と `RuleSetting::Error` をどちらも単なるenabledとして扱っている現状を整理する。
- CLIの終了コードを `error` の有無で決め、`warn` だけなら成功にするかを決める。
- VSCode diagnostics の `DiagnosticSeverity.Warning` / `Error` を出し分ける方針を決める。
- docsとtestsを更新する。

完了条件:

- `warn` / `error` の意味がdocsと実装で一致している。
- 既存ruleを大きく書き換えずにseverityを扱える。
- CLI integration testで `warn` の挙動が固定されている。

範囲外:

- autocorrect
- ruleごとの細かいcategory taxonomy
- LSP implementation

結果:

- `Diagnostic` に `DiagnosticSeverity` を追加した。
- `erbfmt.json` の `warn` / `error` を内部diagnostic severityへ反映するようにした。
- 未指定ruleと `recommended` で有効化されたruleは、従来どおり `error` として扱う方針にした。
- CLIは `error` diagnostic があれば失敗し、warningだけなら成功するようにした。
- CLIのwarning出力は `warning:` prefix を付け、既存のerror出力形式は維持した。
- VSCode wrapperは `warning:` prefix を `DiagnosticSeverity.Warning` として表示するようにした。
- docsとtestsを更新した。

## Milestone 52

Long ERB command-call wrapping

Status: Done

`formatter.lineWidth` を超えた standalone ERB tag のうち、安全に読める Ruby
command-style method call だけを複数行へ折りたたみます。

対象:

- `<%= link_to "Edit", edit_user_path(user), class: "button" %>`
- `<%= form_with model: user, url: user_path(user) do |form| %>`
- `<% tag.div class: "card", data: { controller: "profile" } %>`

折りたたみ方:

```erb
<%=
  link_to(
    "Edit",
    edit_user_path(user),
    class: "button"
  )
%>
```

`do` block suffix は閉じ括弧の後ろへ残します。

```erb
<%=
  form_with(
    model: user,
    url: user_path(user)
  ) do |form|
%>
```

安全側に倒すもの:

- `if`, `unless`, `case`, `when`, `elsif`, `else`, `end` などの control-flow
- 三項演算子、演算子中心の式、代入、複数statement
- top-level comma がない call
- 文字列や括弧の対応を安全に読めない Ruby code
- inline context の隣接ERB output

やること:

- Ruby AST は導入せず、単純な command-call 判定だけを小さな formatter helper に分離する。
- top-level comma の検出では `()`, `[]`, `{}`, quote 内部を分割しない。
- 解析できない場合は現状どおり ERB marker だけを展開し、Ruby code は保持する。
- formatter unit test と CLI integration test で折りたたみ対象 / 非対象を固定する。
- docs に、対応範囲と安全側fallbackを明記する。

完了条件:

- `link_to ...`, `form_with ... do |form|`, `tag.div ...` の代表例を折りたためる。
- control-flow や複雑な式を誤って書き換えない。
- 既存のHTML formatting、空行保持、format-sensitive tag保持に影響しない。

範囲外:

- Ruby AST parsing
- RuboCop完全互換
- 既存の括弧付きRuby callの全面再整形
- Ruby semantic analysis

結果:

- Ruby command-call 折りたたみ helper を formatter 本体から分離して追加した。
- `link_to ...`, `form_with ... do |form|`, `tag.div ...` の代表例を折りたためるようにした。
- `()`, `[]`, `{}`, quote 内部の comma は top-level argument split の対象外にした。
- `if` などのcontrol-flow、top-level comma のないcall、対応が崩れたRuby codeは折りたたまないようにした。
- CLIの `formatter.lineWidth` test と formatter unit testで挙動を固定した。
- 実テンプレート監査fixtureのsnapshotを新しい折りたたみ結果へ更新した。

## Milestone 53

Formatter ignore design

Status: Done

`erbfmt-ignore` のformatter対応を設計します。

やること:

- formatter ignore を next-line に限定するか、node/subtree単位にするか決める。
- mixed AST に source range を持たせる必要があるか確認する。
- HTML comment / ERB comment のどちらで指定できるか決める。
- formatter ignoreがlint ignoreと衝突しない構文を整理する。
- docsと小さなfixtureで期待する挙動を固定する。

完了条件:

- formatter ignore の最小仕様がdocsにある。
- 実装に必要なAST/source range変更が整理されている。
- すぐ実装に入れるテストケースが決まっている。

範囲外:

- block-level formatter disable / enable
- unused ignore directive reporting
- Ruby AST parsing

結果:

- formatter ignore は次の物理行から始まる、同じ親の次の非空白AST node/subtreeを対象にすると決めた。
- directive は `erbfmt-ignore format` とし、HTML commentとERB commentの両方を対象にした。
- formatと全lint診断を同時に抑制する場合は `erbfmt-ignore all` を使う方針にした。
- target subtreeの物理行を原文からbyte-for-byteで保持し、判定が曖昧な場合は通常formatする方針にした。
- inline fragment、HTML attribute単位、block-level disable/enableは初期範囲外にした。
- mixed AST nodeへのabsolute byte range、parser frameの開始/終了range、formatterへの原文引き渡しが必要と整理した。
- ignored raw sourceのline endingを守るため、formatter最後の一括line ending変換も整理が必要と確認した。
- ERB commentは現在通常のERB codeとしてtokenizeされるため、明示的なcomment tokenが必要と確認した。
- [FormatterIgnoreDesign.md](FormatterIgnoreDesign.md) と `samples/formatter-ignore-next.html.erb` を追加した。

## Milestone 54

Formatter ignore implementation

Status: Done

Milestone 53で決めた最小仕様に沿って、formatter ignoreを実装します。

やること:

- lexerにERB comment tokenを追加し、`<%# ... %>`を原形のまま扱えるようにする。
- mixed AST nodeへoptionalなabsolute byte rangeを持たせる。
- HTML elementとERB blockの開始から終了までのrangeをparserで構築する。
- lint / formatterで共有できるignore directive parserを用意する。
- formatterへ元sourceを渡し、対象subtreeの物理行を原文のまま出力する。
- `samples/formatter-ignore-next.html.erb` を使ったunit / CLI / idempotency testを追加する。

完了条件:

- HTML commentとERB commentの両方から次のnode/subtreeをformat対象外にできる。
- ignored subtreeはindent、内部whitespace、line endingを含めて保持される。
- ignored subtreeの前後は通常どおりformatされる。
- lint ignoreの既存挙動にregressionがない。
- 曖昧なdirectiveやinline targetは通常formatへ安全にfallbackする。

範囲外:

- block-level formatter disable / enable
- HTML attribute単位のignore
- unused ignore directive reporting
- Ruby AST parsing

結果:

- lint / formatterで共有するignore directive parserを追加した。
- formatと全lint診断を同時に抑制する `erbfmt-ignore all` を追加した。
- lexerとmixed ASTにERB commentを追加し、`<%# ... %>` markerを保持するようにした。
- `parse_spanned` がmixed AST node/subtreeをabsolute byte range付きで返すようにした。
- HTML element、standalone ERB tag、ERB control-flow blockをsubtree単位でformat対象外にできるようにした。
- ignored subtreeのindent、内部whitespace、line endingを元sourceから保持するようにした。
- directiveの直後にblank lineがある場合など、対象を安全に特定できないときは通常formatへfallbackするようにした。
- formatter unit test、CLI integration test、fixture idempotency testを追加した。
- lint ignoreの既存挙動を共有parserへ移し、regression testを維持した。

## Milestone 55

Ruby gem wrapper design

Status: Done

Rust formatter engineを維持したまま、Ruby / Rails projectから導入しやすい薄いgem wrapperを設計します。

やること:

- source build gemとplatform-specific prebuilt gemのどちらから始めるか決める。
- `erbfmt` executableの解決方法とRust binaryの配置場所を決める。
- gem versionとRust crate / GitHub Release versionの対応方針を決める。
- Ruby LSPやBundlerと同じprojectで使う場合の導入手順を整理する。
- gemspec、Gemfile、Ruby wrapperの最小構成をdocsへまとめる。

完了条件:

- 初期gemの配布形態とplatform supportが決まっている。
- Rust engineを二重実装しないwrapper境界が明文化されている。
- 次Milestoneでscaffoldを作れるfile構成とtest方針が決まっている。

範囲外:

- RubyGems.orgへのpublish
- Rubyによるformatter再実装
- Rails semantic analysis

結果:

- 初期配布はsource build gemではなく、同名・同versionのplatform-specific gemにすると決めた。
- gem executableはRuby scriptとし、同梱したRust binaryを `Kernel.exec` するだけの薄いlauncherにした。
- install時download、PATH fallback、Rubyによるformatter APIは持たない方針にした。
- `x86_64-linux-gnu`, `x86_64-darwin`, `arm64-darwin`, `x64-mingw-ucrt` を初期gem platform候補にした。
- gem versionは公開時にRust crate / CLI / tag / GitHub Releaseと完全一致させる方針にした。
- development gem versionはRubyGems表記の `0.0.0.dev` を使う方針にした。
- Ruby LSP add-onにはせず、Gemfileのdevelopment dependencyとして `require: false` で共存させる方針にした。
- [RubyGem.md](RubyGem.md) にfile構成、binary解決、build、test、versioningを整理した。

## Milestone 56

Ruby gem wrapper scaffold

Status: Done

Milestone 55の設計に沿って、publishしないローカルgem scaffoldを作ります。

やること:

- `packages/ruby` にgemspec、Gemfile、Rakefile、Ruby launcher、version moduleを追加する。
- `ERBFMT_BINARY` とstaged `libexec` binaryの解決を実装する。
- launcherが引数、stdio、signal、exit statusをRust binaryへ渡すようにする。
- Rust debug binaryを使うunit / integration testを追加する。
- platformとbinary pathを指定して `.gem` をbuildできるtaskを追加する。
- CIへRuby wrapperのtest jobを追加するか、追加に必要な残課題を整理する。

完了条件:

- `ERBFMT_BINARY=target/debug/erbfmt` でRuby launcherからCLIを実行できる。
- local platform向けgemをbuildし、isolated `GEM_HOME` へinstallして `erbfmt --version` を確認できる。
- Rust engine以外にformatting logicを持たない。
- gemはpublishされていない。

範囲外:

- RubyGems.orgへのpublish
- release credential / MFA設定
- unsupported platform向けsource build fallback
- GitHub Release automation

結果:

- `packages/ruby` にgemspec、Gemfile、lockfile、Rakefile、Ruby launcher、testを追加した。
- `ERBFMT_BINARY` overrideとstaged `libexec/erbfmt-bin` の解決を実装した。
- Ruby launcherは `Kernel.exec` だけを使い、引数、stdio、signal、exit statusをRust binaryへ渡す構成にした。
- Ruby 3.4 / Bundler 2.6でunit / integration testを追加した。
- Cargoの `0.0.0-dev` とgemの `0.0.0.dev` のversion consistency checkを追加した。
- `rake gem:verify` で `x86_64-linux-gnu` gemをbuildし、isolated `GEM_HOME`へinstallして `erbfmt --version` を確認した。
- CIにRust binary buildと `gem:verify` を行うRuby Wrapper jobを追加した。
- gemはRubyGems.orgへpublishしていない。

## Milestone 57

Cross-platform Ruby gem packaging

Status: Done

release binary matrixから4platformのRuby gem artifactを作り、publish前のpackage検証を自動化します。

やること:

- Rust targetとRubyGems platformの対応をworkflow matrixへ追加する。
- Linux、Intel / Apple Silicon macOS、Windows向けgemをbuildする。
- 各gemに正しいbinary名とplatform metadataが入っていることを確認する。
- isolated gem installと `erbfmt --version` を各runnerで実行する。
- `.gem` artifactをGitHub Actionsへuploadする。
- Rust archiveとgemが同じversion / commitから作られたことを検証する。

完了条件:

- 4platformの `.gem` artifactがworkflowで作られる。
- 各native runnerでinstallとCLI実行が成功する。
- RubyGems.orgへpublishせずpackage内容を確認できる。

範囲外:

- RubyGems.orgへのpublish
- release credential / MFA設定
- source build fallback
- musl / Linux arm64 gem

結果:

- release workflowのRust target matrixへ対応するRubyGems platformを追加した。
- 4platformすべてでRuby 3.4 / Bundlerをsetupし、release binaryからgemをbuildするようにした。
- gemのplatform、version、同梱binary名をinstall前に検証するようにした。
- 各native runnerでisolated `GEM_HOME`へinstallし、`erbfmt --version`を実行するようにした。
- standalone archiveとgemを同じjob、commit、release binaryから作り、同じworkflow artifactへuploadするようにした。
- Bundler lockfileをLinux、Intel / Apple Silicon macOS、Windows UCRTで利用できる構成にした。
- RubyGems.orgへのpublishは行っていない。
- manual workflowで4platformのnative install、CLI実行、artifact uploadが成功した。

## Milestone 58

Cross-platform Ruby gem workflow verification

Status: Done

Milestone 57で追加したrelease matrixをGitHub Actions上で実行し、実際のpackageを確認します。

やること:

- `Release Binaries` workflowをmanual実行する。
- 4platformのjobがgem build、install、CLI実行まで成功することを確認する。
- artifact内のstandalone archive、checksum、gem名を確認する。
- runner固有のRubyGems / Bundler差異があれば修正する。

完了条件:

- 4platformすべてのworkflow jobが成功している。
- 4種類のplatform-specific gem artifactを取得できる。
- public publish前に残る課題がRoadmapへ記録されている。

範囲外:

- RubyGems.orgへのpublish
- GitHub Releaseの自動作成
- release credential / MFA設定

結果:

- run `27865822697` で4platformすべてがgem verification stepまで進むことを確認した。
- Unix runnerではworkflow用の `ERBFMT_BINARY` がinstalled gem実行時にも残り、同梱binaryより優先されていた。
- Windows runnerではtest fixtureのbinary名に `.exe` がなく、実行可能判定に失敗していた。
- isolated gem実行時のoverride解除とWindows用test binary名を修正した。
- run `27866499432` でLinux、Intel / Apple Silicon macOS、Windowsの4jobがすべて成功した。
- 各native runnerでgem build、isolated install、`erbfmt --version`、archive作成、artifact uploadが成功した。
- 4つのstandalone archiveでchecksum一致とbinary、`README.md`、`LICENSE.txt`の同梱を確認した。
- 4つのgemでplatform metadataと `libexec/erbfmt-bin` / `erbfmt-bin.exe` の同梱を確認した。
- RubyGems.orgやGitHub Releaseへのpublishは行っていない。

## Milestone 59

Ruby gem release rehearsal

Status: Done

公開versionを付ける前に、version更新からpackage確認までのrelease手順を通しで検証します。

やること:

- Cargo、Ruby gem、VSCode extensionのversion整合ルールをrelease taskへ反映する。
- development version固有のRubyGems activation補助を使わず、stable version gemを検証する方法を用意する。
- tagまたはrelease commitからworkflowを起動する手順を整理する。
- `0.1.0`相当の一時的なpackage rehearsalを行い、生成物名とCLI versionを確認する。
- rehearsalでrepositoryのdevelopment versionを変更したままにしない。

完了条件:

- version bumpから4platform artifact確認までの手順が文書化されている。
- stable version gemのinstallとCLI実行をpublishせず検証できる。
- release時に更新するfileと実行するcommandが一意に決まっている。

範囲外:

- RubyGems.orgへのpublish
- GitHub Releaseの作成
- release credential / MFA設定

結果:

- `scripts/version.rb set / verify` を追加し、Cargo、Ruby gem、VSCode extensionと各lockfileを一括管理するようにした。
- VSIX filenameを記載する英日docsも同じversion操作で更新するようにした。
- version scriptのisolated testを追加し、CIとRuby Rake taskから実行するようにした。
- release workflowへ任意の `rehearsal_version` inputを追加した。
- 一時copyだけを `0.1.0`へ更新し、release Rust binaryとstable Linux gemのbuild、install、CLI実行に成功した。
- 同じ一時copyから `erbfmt-vscode-0.1.0.vsix` をpackageできることを確認した。
- stable gemの検証ではdevelopment version用のRubyGems activation補助が使われないことを確認した。
- 元repositoryが `0.0.0-dev`のままであることを確認した。
- tag / release commitではinputを使わず、checked-out fileのversionからartifactを作る手順を文書化した。
- 4platform stable-version rehearsalが成功した。

## Milestone 60

Cross-platform stable-version rehearsal

Status: Done

Milestone 59で追加したworkflow inputを使い、`0.1.0`相当の4platform artifactを検証します。

やること:

- `main`から `rehearsal_version=0.1.0` で `Release Binaries` workflowを実行する。
- 4platformのarchive、checksum、gemが `0.1.0`で生成されることを確認する。
- 各native runnerでstable gemのinstallとCLI version確認が成功することを確認する。
- workflow後も `main`が `0.0.0-dev`のままであることを確認する。

完了条件:

- 4platformすべてのrehearsal jobが成功している。
- standalone archiveとgemのversion、platform、同梱binaryが正しい。
- repositoryのdevelopment versionに差分が残っていない。

範囲外:

- RubyGems.orgへのpublish
- GitHub Releaseの作成
- release tagの作成

結果:

- `main`のcommit `ea5c4dd`から `rehearsal_version=0.1.0` を指定してrun `27876841135`を実行した。
- Linux、Intel / Apple Silicon macOS、Windowsの4jobがすべて成功した。
- 各runnerで一時version更新、version整合検証、release build、stable gem install、`erbfmt --version`が成功した。
- 4つのstandalone archiveとchecksumが `0.1.0`名で生成され、checksum一致を確認した。
- 各archiveに対象platformのbinary、`README.md`、`LICENSE.txt`が含まれることを確認した。
- 4つのgemがversion `0.1.0`、想定したRubyGems platform、正しい `libexec` binary名を持つことを確認した。
- workflow artifact以外のpublish、GitHub Release、release tag作成は行っていない。
- workflow後もremote `main`とlocal repositoryのversionは `0.0.0-dev`で、作業ツリーにversion差分がないことを確認した。

## Milestone 61

First public release readiness audit

Status: Done

実際のversion bumpやtag作成の前に、初回releaseを開始できる状態か最終監査します。

やること:

- [FirstRelease.md](FirstRelease.md) の手順と現在のworkflow、artifact、READMEを照合する。
- 公開対象をstandalone binary、checksum、未公開gem artifactのどこまでにするか最終確認する。
- repository metadata、license、version source、known limitationsを確認する。
- release notesに必要な利用方法、対応platform、制約を整理する。
- blockerとrelease後へ送る課題を分離し、go / no-goを記録する。

完了条件:

- 初回releaseの公開物と手順に矛盾がない。
- release前に必須の作業と、release後でもよい作業が明確になっている。
- version bump、tag、publishを始めてよいか判断できる。

範囲外:

- release commit / tagの作成
- GitHub ReleaseやRubyGems.orgへのpublish

進行状況:

- ユーザー判断により最終監査より先にrepository全体を `0.1.0`へ更新した。
- Cargo、Ruby gem、VSCode extension、各lockfile、VSIX記載のversion整合を確認した。
- release tagとpublishはまだ行っていない。

結果:

- 初回公開物を4platformのstandalone binary、checksum、platform-specific gem、
  thin VSIXに確定した。
- standalone binary、checksum、VSIXはGitHub Release assetとして公開する。
- gemは同じtagからworkflowで生成したartifactをRubyGems.orgへ手動公開する。
- VSIXはRust binaryを同梱せず、別途installまたは設定された `erbfmt`を利用する。
- crates.io、npm、VSCode Marketplace、GitHub Packagesは初回releaseの範囲外とした。
- `0.1.0` releaseを先に完了し、parenthesized ERB call wrappingは `0.1.1`
  候補として別branchに維持する方針とした。

## Milestone 62

`v0.1.0` ecosystem packaging preparation

Status: Done

決定した公開物を同じrelease refから再現できるようにし、公開直前の手順を固定します。

やること:

- `Release Binaries` workflowで4platformのbinary、checksum、gemに加えてVSIXを作る。
- rehearsal versionでもbinary / gem / VSIXのversionを揃える。
- GitHub Releaseへの添付物とRubyGems.orgへの手動publish手順を文書化する。
- VSIXがbinaryを同梱せず、別途CLI installが必要であることを明記する。

完了条件:

- workflowが `erbfmt-vscode-${version}.vsix`をartifactとしてuploadする。
- release docs、distribution docs、Ruby gem docsの公開範囲が一致している。
- actual tag / publishを行わずに、次のrelease milestoneへ進める状態になっている。

範囲外:

- `v0.1.0` tag作成
- GitHub Release公開
- RubyGems.orgへのpush
- VSCode Marketplace公開

結果:

- `Release Binaries` workflowに独立したVSCode jobを追加した。
- VSCode jobも `rehearsal_version`を適用し、repository全体のversion整合を検証する。
- VSCode extensionのtestとpackageを実行し、生成したVSIXをworkflow artifactとして
  uploadするようにした。
- localでversion test、version verify、VSCode test、VSIX packageが成功した。
- 初回公開物の配布先、artifact名、RubyGems.orgへの手動push順序を文書間で統一した。
- actual tag、GitHub Release、RubyGems.org publishはMilestone 63へ分離した。

## Milestone 63

`v0.1.0` tag and ecosystem release

Status: Planned

検証済みcommitへtagを付け、binary、gem、VSIXを初めて公開します。

やること:

- cleanな `main`でrelease verificationを完走する。
- verified commitへannotated tag `v0.1.0`を付けてpushする。
- tagから `Release Binaries` workflowを実行する。
- binary、checksum、VSIXをdraft GitHub Releaseへ添付する。
- workflowで検証した4platform gemをRubyGems.orgへpushする。
- clean environmentでbinaryとgemをsmoke testしてからGitHub Releaseを公開する。

完了条件:

- GitHub Release `v0.1.0`が公開されている。
- 4platformのstandalone binaryとchecksumを取得できる。
- RubyGems.orgから対応platformの `erbfmt`をinstallできる。
- VSIXをGitHub Releaseから取得でき、別途導入したCLIで動作する。

範囲外:

- crates.io / npm / GitHub Packages
- VSCode Marketplace
- VSCode binary download / cache

## 後で考えること

- VSCode Marketplace publishing
- GitHub Release automation
- binary download / cache logic
- Tree-sitter integration
- Biome integration
- LSP

これらは、Rust formatter が実テンプレートに対して十分安定し、binary release の
境界が固まってから進めます。
