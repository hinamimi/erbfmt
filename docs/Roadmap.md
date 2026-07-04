# Roadmap

この文書は、erbfmt の「次にどこへ進めるか」を短く確認するための
ロードマップです。

完了済みの細かい作業ログはここには残しません。release 手順、配布方針、
設定仕様、lint rule の詳細は、それぞれ専用ドキュメントを参照します。

## 方針

erbfmt は `*.html.erb` 向けの formatter / linter です。

目標は、Prettier、Biome、dprint に近い体験を ERB template に提供することです。

- formatter / linter engine は Rust に置く。
- VSCode extension、Ruby gem、将来の npm package は薄い wrapper として扱う。
- Ruby AST parsing は formatter から独立した基盤として段階的に育てる。
- HTML / ERB の構造を安全に扱い、semantic を壊しそうな場合は無理に整形しない。
- 実テンプレートでの安全性を、見た目のきれいさより優先する。

## 現在のベースライン

現在の公開版は `v0.2.1` です。

実装済み:

- Rust CLI
- `--write`, `--check`, `--lint`
- directory input の再帰探索
- multi-file 実行時の file-level error 継続
- `erbfmt init`
- `erbfmt.json`
- `files.includes` による対象 / 非対象ファイル指定
- ERB lexer
- lightweight HTML tokenizer
- HTML-aware mixed parser
- `parser.allowHtmlOptionalClosingTags` による HTML optional closing tag の許容
- mixed AST-driven formatter
- ERB block / branch formatting
- HTML tag attribute wrapping
- parenthesized Ruby helper call wrapping
- 安全に扱える command-style Ruby helper call wrapping
- Ruby AST-assisted method call wrapping
- Ruby receiver method call with brace block suffix parsing for helper wrapping
- Ruby helper wrapping with `if` / `unless` modifier suffix preservation
- Ruby quoted label keyword argument parsing for helper wrapping
- Ruby nested helper call wrapping inside keyword arguments
- Ruby assignment / operator expression AST
- Ruby modifier / safe navigation / index access AST
- Ruby ternary / range AST
- Ruby splat / constant path / percent literal AST
- top-level multi-statement ERB code の preservation
- inline whitespace を壊さない formatter safety
- `<%- %>`, `<% -%>`, `<%== %>` の marker preservation
- formatter ignore directive
- basic syntax lint rules
- HTML5 self-closing / deprecated tag lint rules
- duplicate / boolean / quoted attribute lint rules
- list / table / paragraph nesting lint rules
- unmatched HTML close tag diagnostics
- pretty lint output
- `--lint-format json` による machine-readable diagnostics
- hosted `commands.md` による AI agent 向け command reference
- VSCode `html-erb` language id
- VSCode TextMate syntax highlighting
- VSCode document formatter wrapper
- VSCode diagnostics wrapper
- VSCode ERB-safe comment toggling
- Ruby gem wrapper
- RubyGems.org publication
- VSCode Marketplace publication
- GitHub Release 向け binary / gem / VSIX packaging
- GitHub Pages documentation

主な文書:

- [Configuration.md](Configuration.md): `erbfmt.json` の仕様
- [LintRules.md](LintRules.md): lint rule の一覧
- [Ignore.md](Ignore.md): ignore directive
- [Release.md](Release.md): release verification と versioning
- [Distribution.md](Distribution.md): binary distribution strategy
- [RubyGem.md](RubyGem.md): Ruby gem wrapper
- [VSCode.md](VSCode.md): VSCode extension
- [AI.md](AI.md): AI agent 向け command usage
- [commands.md](commands.md): hosted commands.md の source
- [Development.md](Development.md): local development workflow

主な sample:

- `samples/sample.html.erb`: formatter demo
- `samples/lint-next.html.erb`: lint rule sample
- `samples/stability.html.erb`: formatting stability sample
- `samples/formatter-audit.html.erb`: formatter audit sample
- `samples/formatter-edge-cases.html.erb`: edge-case sample
- `samples/real-template-audit.html.erb`: Rails寄りの実テンプレート sample
- `samples/html-parse-errors.html.erb`: HTML parse diagnostic sample

## 現在の制約

- Ruby AST は helper call wrapping の安全判定に使い始めている。
- Ruby expression の folding は、構文的に安全に分割できる範囲だけ扱う。
- CSS の `display` や Rails helper の semantic analysis はしない。
- crates.io、npm、Open VSX はまだ公開しない。
- VSCode extension は Rust binary をまだ bundle / download しない。
- AI agent 向けの出力は、stdout/stderr の文字列 parse に頼らず JSON へ寄せる。
- formatter が安全に判断できない領域は、整形しないか lint / diagnostic に寄せる。

## 次にやる Milestone

### Milestone A: v0.2.x release hygiene

`v0.2.1` で RubyGems.org / GitHub Release / VSCode Marketplace の release path は
一通り動いています。今後の patch release を軽く回せるように、手順を短く保ちます。

やること:

- release 後 smoke test の最小コマンドを [Release.md](Release.md) に集約する。
- RubyGems.org publish 後の確認手順を短くする。
- GitHub issue を release と紐付けて close する運用を続ける。
- patch release の判断基準を、bugfix / docs / packaging で分ける。

完了条件:

- 新しい patch release を迷わず作れる。
- RubyGems.org、GitHub Release、VSCode Marketplace の公開状態を短時間で確認できる。
- release docs に古い version 固定の説明が残りにくい。

### Milestone B: Ruby AST foundation

ERB tag 内 Ruby expression を安全に理解するための AST 基盤を育てます。

やること:

- `src/ruby_ast/` の lexer / parser / AST を小さく保つ。
- helper call、keyword argument、array、hash、assignment、operator expression、
  modifier `if` / `unless`、ternary、range、splat、constant path、
  percent literal、safe navigation、index access、top-level multiple statements、
  nested call、`do |param|` suffix を扱う。
- assignment は AST 化するが、formatter 対象からは外す。
- operator expression は AST 化し、method call wrapping の安全判定に使う。
- multiple statements は AST 化するが、formatter 対象からは外す。
- modifier `if` / `unless` は AST 化するが、formatter 対象からは外す。
- ternary は AST 化する。top-level ternary は formatter 対象から外すが、
  helper call の引数として安全に保持できる場合は wrapping 対象にする。
- parse できる / できない Ruby の fixture を追加する。
- formatter 出力生成へ完全接続する前に、AST から wrapping plan を作る設計を決める。

完了条件:

- Rails helper で頻出する simple call を AST 化できる。
- assignment を AST 化しつつ、format candidate ではないと判定できる。
- `&&`, `||`, comparison, arithmetic, unary, grouped expression を AST 化できる。
- modifier `if` / `unless`、safe navigation、index access を AST 化できる。
- ternary と inclusive / exclusive range を AST 化できる。
- `*args` / `**options` / `&block`、`Admin::User`、`%w[]` / `%i[]`
  などを AST 化できる。
- semicolon / top-level newline で分かれた multiple statements を AST 化しつつ、
  format candidate ではないと判定できる。
- 危険な Ruby expression を AST 化せず、明示的に error にできる。
- formatter で使える AST の責務と制約が test で固定されている。

### Milestone C: Ruby wrapping AST integration

Ruby AST を使って、既存の heuristic wrapping を段階的に置き換えます。
現在は、method call を折りたためるかどうかの安全判定に AST を使い、
出力生成は既存の conservative wrapper が担当しています。

やること:

- `render(...)`, `form_with(...)`, `link_to(...)`, `image_tag(...)`,
  `video_tag(...)`, `react_component(...)` の AST-based wrapping を設計する。
- `f.select ... Foo.keys.map { |value| ... }` のような receiver block 付き
  helper argument を安全判定に使う。
- `render ... unless condition` のような modifier suffix 付き helper call を、
  suffix を保持したまま wrapping する。
- `"aria-label": ...` や `"data-action": ...` のような quoted label keyword
  arguments を安全判定に使う。
- `t(..., action: link_to(...))` のような nested helper call を keyword
  argument 内で安全に wrapping する。
- command-style call wrapping が semantic を変えそうな場合は wrapping しない。
- hash / array / keyword arguments の indentation を AST から生成する。
- 既存 `ruby_wrap.rs` の fixture を AST-based formatter plan へ移す。

完了条件:

- long ERB output が `lineWidth` を大きく超えにくくなる。
- 折り返した ERB tag の indentation が一貫する。
- Ruby semantic を変える変換をしない。
- 危険な変換は test fixture で regression として検出できる。

### Milestone D: formatter safety fixture expansion

実テンプレートで design を壊しそうな formatter behavior を fixture に寄せます。

やること:

- inline element と隣接 text の whitespace preservation を追加で検証する。
- `a`, `label`, `button`, `span`, `i`, `svg`, `math`, `pre`, `textarea`,
  `script`, `style`, `contenteditable` 周辺の安全側 behavior を固定する。
- attribute 内 ERB、quoted / unquoted attribute、long class / style の折り返しを検証する。
- safe に整形できないケースは、拒否または元の表現保持に倒す。

完了条件:

- inline whitespace semantic を変える regression が snapshot で検出できる。
- 実テンプレート sample に対する format が idempotent である。
- formatter が壊しそうなケースを Roadmap ではなく test fixture で管理できる。

### Milestone E: linter quality

HTML / ERB の lint を、実用上の発見力と誤検知の少なさの両方で改善します。

やること:

- HTML content model rule の range と message を改善する。
- unquoted attribute、duplicate attribute、invalid nesting の例を増やす。
- ignore directive と lint rule の相互作用を整理する。
- rule severity / enable / disable の設定を見直す。
- VSCode diagnostics で見たときの message を短くする。

完了条件:

- lint error の場所が editor 上で理解しやすい。
- HTML-only rule と ERB-aware rule の責務が分かれている。
- `samples/lint-next.html.erb` が lint rule の代表例として使える。

### Milestone F: VSCode extension UX

VSCode で「入れたら動く」に近づけます。

やること:

- GitHub Release から platform binary を取得する方式を設計する。
- binary cache directory と update policy を決める。
- `erbfmt.command` を使う手動指定との優先順位を決める。
- binary missing / permission error の message を改善する。
- Open VSX 公開が必要か判断する。

完了条件:

- VSIX 導入後に、別途 CLI を入れなくても動かせる道筋がある。
- local checkout 開発と installed extension の binary resolution が混ざらない。
- Marketplace 版の user experience が説明できる。

### Milestone G: AI-friendly interface

Codex / Claude Code などの agent が、Rails view を安全に編集しやすい interface を整えます。

やること:

- `--lint-format json` の schema を安定させる。
- lint / parse error の file、severity、message、line、column を継続的に維持する。
- 複数 file 実行では agent が stream 処理しやすい出力形式にする。
- hosted `commands.md` を短く保ち、Codex / Claude Code から参照しやすくする。
- Codex Skill / Claude Code Skill から呼びやすい command example を整理する。
- formatter ignore / lint ignore の使いどころを agent 向けに説明する。

完了条件:

- agent が `erbfmt --lint --lint-format json` の結果から修正対象を特定できる。
- AI 向け docs を読めば、Rails view 編集後に実行すべき command が分かる。
- 人間向け CLI 出力と agent 向け JSON 出力が役割分担できている。

### Milestone H: remaining package registry strategy

未公開の registry へ進むか判断します。

候補:

- crates.io
- npm
- Open VSX

やること:

- registry ごとの versioning / signing / artifact policy を確認する。
- GitHub Release asset との役割分担を決める。
- npm package を作る場合、binary download wrapper にするかを決める。

完了条件:

- どの registry から公開するか、またはまだ公開しないか判断できる。
- 公開する場合の release checklist がある。

## 後で考えること

- Ruby AST parser の導入
- Tree-sitter integration
- LSP
- Biome integration
- formatter plugin API
- autocorrect
- Rails semantic lint

これらは、Rust formatter が実テンプレートに対して十分安定し、binary release の
境界が固まってから進めます。
