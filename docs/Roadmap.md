# Roadmap

この文書は、erbfmt の「今後どこへ進めるか」を短く確認するための
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

実装済み:

- Rust CLI
- `--write`, `--check`, `--lint`
- `erbfmt init`
- `erbfmt.json`
- `files.includes` による対象 / 非対象ファイル指定
- ERB lexer
- lightweight HTML tokenizer
- HTML-aware mixed parser
- `parser.allowHtmlOptionalClosingTags` によるHTML optional closing tagの許容
- mixed AST-driven formatter
- ERB block / branch formatting
- HTML tag attribute wrapping
- parenthesized Ruby helper call wrapping
- inline whitespace を壊さない formatter safety
- `<%- %>`, `<% -%>`, `<%== %>` の marker preservation
- formatter ignore directive
- basic syntax lint rules
- HTML5 self-closing / deprecated tag lint rules
- duplicate / boolean / quoted attribute lint rules
- list / table / paragraph nesting lint rules
- unmatched HTML close tag diagnostics
- VSCode `html-erb` language id
- VSCode TextMate syntax highlighting
- VSCode document formatter wrapper
- VSCode diagnostics wrapper
- VSCode ERB-safe comment toggling
- platform-specific Ruby gem wrapper
- GitHub Release 向け binary / gem / VSIX packaging
- GitHub Pages documentation

主な文書:

- [Configuration.md](Configuration.md): `erbfmt.json` の仕様
- [LintRules.md](LintRules.md): lint rule の一覧
- [Ignore.md](Ignore.md): ignore directive
- [Release.md](Release.md): release verification と versioning
- [Distribution.md](Distribution.md): binary distribution strategy
- [RubyGem.md](RubyGem.md): platform-specific Ruby gem wrapper
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
- RubyGems.org、crates.io、npm、VSCode Marketplace にはまだ公開しない。
- VSCode extension は Rust binary をまだ bundle / download しない。
- formatter が安全に判断できない領域は、整形しないか lint / diagnostic に寄せる。

## 次にやる Milestone

### Milestone A: v0.1.x release hygiene

GitHub Release だけで配布する前提で、release 作業を安定させます。

やること:

- tag push から draft release が作られる流れを実運用で確認する。
- binary、checksum、gem、VSIX の asset 名を固定する。
- release 後の smoke test 手順を短くする。
- README / docs site / package README の install 説明を揃える。
- 0.1.x patch release の判断基準を明文化する。

完了条件:

- 新しい patch release を迷わず作れる。
- GitHub Release から CLI / gem / VSIX を導入する説明が破綻していない。
- registry に公開しない期間の運用が明確になっている。

### Milestone B: formatter safety hardening

実テンプレートで design を壊しそうな formatter behavior をさらに潰します。

やること:

- inline element と隣接 text の whitespace preservation を追加で検証する。
- `a`, `label`, `button`, `span`, `i`, `svg`, `math`, `pre`, `textarea`,
  `script`, `style`, `contenteditable` 周辺の安全側 behavior を固定する。
- opening / closing tag と children の元の隣接境界を保持する。
- attribute 内 ERB、quoted / unquoted attribute、long class / style の折り返しを検証する。
- safe に整形できないケースは、拒否または元の表現保持に倒す。

完了条件:

- inline whitespace semantic を変える regression が snapshot で検出できる。
- 実テンプレート sample に対する format が idempotent である。
- formatter が壊しそうなケースを Roadmap ではなく test fixture で管理できる。

### Milestone C: Ruby expression wrapping

Ruby AST なしで安全に扱える ERB tag 内 expression wrapping を広げます。

やること:

- `render(...)`, `form_with(...)`, `image_tag(...)`, `video_tag(...)`,
  `react_component(...)` のような parenthesized call を安定して折り返す。
- top-level argument split の安全性を上げる。
- block 付き helper call の `do ... end` marker と indentation を揃える。
- hash / array / keyword arguments の indentation を読みやすくする。
- 解析できない Ruby は無理に折り返さない。

完了条件:

- long ERB output が `lineWidth` を大きく超えにくくなる。
- 折り返した ERB tag の indentation が一貫する。
- Ruby semantic を変える変換をしない。

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
- Marketplace / Open VSX 公開前に必要な metadata を整理する。

完了条件:

- VSIX 導入後に、別途 CLI を入れなくても動かせる道筋がある。
- local checkout 開発と installed extension の binary resolution が混ざらない。
- Marketplace 公開を始めるかどうか判断できる。

### Milestone F: package registry strategy

GitHub Release 配布から、registry 公開へ進むか判断します。

候補:

- RubyGems.org
- crates.io
- npm
- VSCode Marketplace
- Open VSX

やること:

- registry ごとの versioning / signing / artifact policy を確認する。
- GitHub Release asset との役割分担を決める。
- Ruby gem の Bundler 導入体験をさらに短くする。
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
