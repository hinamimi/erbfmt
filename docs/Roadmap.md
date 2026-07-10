# Roadmap

この文書は、erbfmt の今後の実装方針を短く確認するためのロードマップです。
完了済みの細かい作業ログは残さず、次に設計・実装する判断に必要な内容だけを置きます。

関連する詳細は専用ドキュメントを参照します。

- [Configuration.md](Configuration.md): `erbfmt.json`
- [LintRules.md](LintRules.md): lint rule
- [Ignore.md](Ignore.md): ignore directive
- [Release.md](Release.md): release
- [Distribution.md](Distribution.md): 配布方針
- [RubyGem.md](RubyGem.md): Ruby gem
- [VSCode.md](VSCode.md): VS Code extension
- [AI.md](AI.md): AI agent 向け interface
- [Development.md](Development.md): 開発用メモ

## 基本方針

erbfmt は `*.html.erb` 向けの formatter / linter です。

目標は、Prettier、Biome、dprint に近い体験を ERB template に提供することです。

- formatter / linter engine は Rust に置く。
- VS Code extension、Ruby gem、将来の npm package は薄い wrapper として扱う。
- formatter は見た目のきれいさより、rendered HTML の意味を壊さないことを優先する。
- 安全に判断できない Ruby / HTML / ERB は、無理に整形せず保持する。
- 実テンプレートで見つかった危険な変更は fixture にして regression を防ぐ。
- AI agent が使いやすいように、human-readable output と machine-readable output を分ける。

## 現在の制約

- Ruby AST は formatter の安全判定に使い始めているが、Ruby 全体を完全に解析するものではない。
- Ruby expression の folding は、構文的に安全に分割できる範囲だけ扱う。
- CSS の `display` や Rails helper の semantic analysis はしない。
- formatter が安全に判断できない領域は、整形しないか lint / diagnostic に寄せる。
- VS Code extension は、現時点では erbfmt CLI を呼び出す wrapper として扱う。
- crates.io、npm、Open VSX への公開はまだ必須ではない。

## Milestone A: formatter safety

実テンプレートで design を壊しそうな formatter behavior を減らします。

やること:

- inline element と隣接 text の whitespace preservation を fixture で固定する。
- `a`, `label`, `button`, `span`, `i`, `svg`, `math`, `pre`, `textarea`,
  `script`, `style`, `contenteditable` 周辺の保護を維持する。
- attribute 内 ERB、quoted / unquoted attribute、long class / style の折り返しを検証する。
- cross-ERB-block HTML や fragment template は、無理に整形せず拒否または保持に倒す。
- 実プロジェクト audit で見つかった危険な変更を、Roadmap ではなく test fixture に移す。

完了条件:

- inline whitespace semantic を変える regression が test で検出できる。
- 実テンプレート sample に対する format が idempotent である。
- safe に整形できるケースと保持するケースの境界が test で説明できる。

## Milestone B: Ruby AST based wrapping

Ruby AST を使って、ERB tag 内の helper call folding を安全に広げます。

やること:

- `render`, `form_with`, `link_to`, `image_tag`, `video_tag`,
  `react_component`, `tag.*`, `content_tag` などの frequent helper call を安全に fold する。
- receiver method call、keyword argument、array、hash、nested helper call を安全判定に使う。
- modifier suffix、safe navigation、constant path、percent literal、splat 系引数を保持する。
- assignment、multiple statements、top-level ternary、unsupported block parameter、
  heredoc、slash regexp literal などは fold しない。
- 出力生成は full AST pretty-printer にせず、当面は AST gate + conservative formatter とする。

完了条件:

- long ERB output が `lineWidth` を大きく超えにくくなる。
- 折り返した ERB tag の indentation が一貫する。
- Ruby semantic を変える変換をしないことが fixture で固定されている。

## Milestone C: parser / syntax architecture

HTML、ERB、Ruby、template parser の責務を整理し、今後の拡張で迷いにくくします。

現方針:

- `src/syntax/erb/`: ERB lexer
- `src/syntax/html/`: lightweight HTML tokenizer / tag utilities
- `src/syntax/template/`: HTML + ERB mixed parser
- `src/syntax/ruby/`: Ruby AST parser
- `src/formatter/`: formatting policy and rendering
- `src/linter/`: lint rule orchestration
- `src/ignore/`: shared ignore directive parsing

やること:

- `src/syntax/template/` の parser body を必要に応じて分割する。
- `src/linter/html_rules.rs` を rule domain ごとに分割する。
- `src/main.rs` が肥大化したら CLI orchestration を `src/cli/` に切り出す。

完了条件:

- syntax layer は formatter / linter の policy を持たない。
- formatter / linter は syntax layer の AST / token を消費するだけにする。
- 1000 行を超える巨大ファイルを作らない。

## Milestone D: linter quality

HTML / ERB の lint を、実用上の発見力と誤検知の少なさの両方で改善します。

やること:

- style / normalization 系の rule は、可能なら assist action として提供する。
- HTML content model rule の range と message を改善する。
- unquoted attribute、duplicate attribute、invalid nesting の例を増やす。
- `input`, `a`, `button` など頻出tagから、unknown / typo HTML attribute
  lintを段階的に追加する。
- ignore directive と lint rule の相互作用を整理する。
- rule severity / enable / disable の設定を見直す。
- VS Code diagnostics で見たときの message を短くする。

完了条件:

- lint error の場所が editor 上で理解しやすい。
- HTML-only rule と ERB-aware rule の責務が分かれている。
- `samples/lint-next.html.erb` が lint rule の代表例として使える。

## Milestone E: AI-friendly interface

Codex / Claude Code などの agent が、Rails view を安全に編集しやすい interface を整えます。

やること:

- `--lint-format json` の schema を安定させる。
- parse / lint / format error の file、severity、message、line、column を維持する。
- 複数 file 実行では agent が stream 処理しやすい出力形式を検討する。
- hosted `commands.md` を短く保ち、agent が参照しやすい command example を整理する。
- formatter ignore / lint ignore の使いどころを agent 向けに説明する。

完了条件:

- agent が JSON output から修正対象を特定できる。
- AI 向け docs を読めば、Rails view 編集後に実行すべき command が分かる。
- 人間向け CLI 出力と agent 向け JSON 出力が役割分担できている。

## Milestone F: assist actions

lint の一部を、安全な編集候補として提供します。

方針:

- linter は問題を知らせる。
- assist は安全に適用できる `TextEdit` を返す。
- correctness 系の rule は linter に残す。
- style / normalization 系は assist に寄せる。
- correctness かつ機械的に直せるものは、linter と assist の両方で扱う。

現在の action:

- `html.useDoubleQuotedAttributes`
- `html.fixSelfClosingTags`
- `html.simplifyBooleanAttributes`
- `erb.removeEmptyTags`

やること:

- VS Code code action から assist を呼べるようにする。
- `--assist-format json` の schema を AI / editor 向けに安定させる。
- linter diagnostic と assist action の対応関係を docs にまとめる。

完了条件:

- `erbfmt --assist` で候補を確認できる。
- `erbfmt --assist --write` で safe action を適用できる。
- linter から移すべき rule と残すべき rule の境界が説明できる。

## Milestone G: editor / distribution UX

VS Code extension と配布経路を、普段使いしやすい状態に近づけます。

やること:

- VS Code extension で platform binary を bundle / download するか判断する。
- `erbfmt.command` による `bundle exec erbfmt` などの手動指定を維持する。
- binary missing / permission error の message を改善する。
- crates.io、npm、Open VSX へ公開するか判断する。

完了条件:

- Rails project では RubyGems + Bundler で導入しやすい。
- VS Code では CLI の場所を説明しやすい。
- 新しい registry に公開する場合の release checklist がある。

## 後で考えること

- full Ruby pretty-printer
- Tree-sitter integration
- LSP
- Biome integration
- formatter plugin API
- autocorrect / assist action
- Rails semantic lint
