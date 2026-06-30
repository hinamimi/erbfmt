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
- Ruby AST parsing はまだ前提にしない。
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

- Ruby AST はまだ使っていない。
- Ruby expression の folding は、構文的に安全に分割できる範囲だけ扱う。
- CSS の `display` や Rails helper の semantic analysis はしない。
- crates.io、npm、Open VSX はまだ公開しない。
- VSCode extension は Rust binary をまだ bundle / download しない。
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

### Milestone B: Ruby wrapping safety hardening

Ruby AST なしで安全に扱える ERB tag 内 expression wrapping を広げつつ、
危険な Ruby は確実に保持します。

やること:

- 複数 statement、assignment、modifier `if` / `unless`、method chain、block 渡しの
  危険パターンを fixture 化する。
- command-style call wrapping が semantic を変えそうな場合は wrapping しない。
- `render(...)`, `form_with(...)`, `link_to(...)`, `image_tag(...)`,
  `video_tag(...)`, `react_component(...)` の parenthesized call を安定させる。
- top-level argument split の安全性を上げる。
- hash / array / keyword arguments の indentation を読みやすくする。

完了条件:

- long ERB output が `lineWidth` を大きく超えにくくなる。
- 折り返した ERB tag の indentation が一貫する。
- Ruby semantic を変える変換をしない。
- 危険な変換は test fixture で regression として検出できる。

### Milestone C: formatter safety fixture expansion

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

### Milestone D: linter quality

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

### Milestone E: VSCode extension UX

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

### Milestone F: remaining package registry strategy

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
