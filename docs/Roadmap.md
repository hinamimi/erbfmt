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

参照する主な文書:

- [Configuration.md](Configuration.md): `erbfmt.json` の仕様
- [Release.md](Release.md): release verification と versioning
- [Distribution.md](Distribution.md): binary distribution strategy
- [VSCode.md](VSCode.md): VSCode extension と local package

参照する主な sample:

- `samples/sample.html.erb`: 意図的に未整形の formatter demo
- `samples/lint-next.html.erb`: lint rule sample
- `samples/stability.html.erb`: formatting stability sample
- `samples/formatter-audit.html.erb`: real-template formatter audit sample
- `samples/formatter-edge-cases.html.erb`: focused formatter edge-case sample

既知の制約:

- Ruby AST parsing はまだしない。
- npm package / Ruby gem はまだない。
- VSCode extension はまだ公開していない。
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

- `Release Binaries` workflow を手動実行し、Linux x64 と Windows x64 の
  artifact が作成されることを確認した。
- 成功した artifact 名は [Distribution.md](Distribution.md) の命名規則と一致していた。
- Linux x64 と Windows x64 の `.sha256` は検証に成功した。
- archive の中身は binary、`LICENSE.txt`、`README.md` で揃っていた。
- macOS arm64 は package step で `sha256sum` がなく失敗したため、
  workflow に `shasum -a 256` fallback を追加した。
- `actions/upload-artifact` の Node.js 20 deprecation annotation に対応するため v5 に更新した。
- macOS x64 は失敗原因が判明したあとに残りrunをキャンセルした。
- `scripts/package-binary.sh` は現hostで成功した。
- 修正後の workflow は次回 push 後に再度 `workflow_dispatch` で確認する。

## Milestone 38

Formatter real-template audit pass

Status: Next

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

## Milestone 39

Lint rule design pass

Status: Planned

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

## Milestone 40

VSCode binary resolution UX

Status: Planned

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

## Milestone 41

First public release planning

Status: Planned

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

## 後で考えること

- npm package
- Ruby gem
- VSCode Marketplace publishing
- GitHub Release automation
- binary download / cache logic
- Tree-sitter integration
- Biome integration
- LSP

これらは、Rust formatter が実テンプレートに対して十分安定し、binary release の
境界が固まってから進めます。
