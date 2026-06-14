# Roadmap

## Project Direction

erbfmt aims to provide a Prettier/Biome-like formatting experience for
`*.html.erb` templates.

The formatter treats ERB as HTML plus control-flow markers. Ruby expressions are
preserved as text; Ruby semantic analysis remains out of scope for now.

## Current Baseline

Implemented:

- Rust CLI for single-file and multi-file use
- `--write`, `--check`, and `--lint`
- VSCode workspace format-on-save setup
- ERB lexer
- Lightweight HTML tokenizer
- HTML-aware mixed parser
- Mixed AST-driven formatter
- ERB block formatting for `if`, `unless`, `case`, `do`, `begin`, and `end`
- ERB branch formatting for `else`, `elsif`, `when`, `rescue`, and `ensure`
- Output ERB do-block formatting for helpers such as
  `<%= form_with ... do |form| %>`
- Attribute ERB output handling inside HTML tags
- Basic syntax lint rules
- Snapshot tests with `insta`
- CLI integration tests that invoke the compiled binary
- Release checklist and local install documentation
- VSCode workspace language association for `*.html.erb`
- Thin VSCode extension scaffold
- Line/column diagnostics for lexer and ERB parser errors
- `erbfmt.json` formatter and linter configuration
- `formatter.lineWidth` wrapping for long HTML tags

Reference samples:

- `samples/sample.html.erb`: ordinary formatting sample
- `samples/lint-next.html.erb`: lint rule sample
- `samples/stability.html.erb`: formatting stability sample
- `samples/formatter-audit.html.erb`: real-template formatter audit sample

Known constraints:

- Ruby code is not parsed as Ruby AST.
- Formatting currently normalizes most non-meaningful whitespace.
- Lint rule diagnostics do not yet include line/column spans.
- Distribution wrappers are not packaged yet.
- The VSCode extension scaffold is not published yet.

## Immediate Focus

The Rust binary is documented enough for local pre-release use, the workspace
has a thin VSCode extension scaffold, syntax diagnostics include source
locations, formatter/linter behavior can be configured through `erbfmt.json`,
and real-template formatter behavior is covered by an audit fixture. The next
milestone should prepare the extension for local packaging or add editor-host
tests.

### Milestone 19

Release and Packaging Preparation

Status: Done

Prepare the Rust CLI for local and pre-release distribution.

Target work:

- Expose binary metadata through `--version` and useful `--help` text.
- Document `cargo install --path .` for local installation.
- Add release build notes.
- Decide the canonical binary name and package naming conventions.
- Document which files are part of release verification.
- Keep npm package and Ruby gem as thin future wrappers around the Rust binary.

Acceptance:

- `erbfmt --version` reports the crate version.
- README and README_ja describe local installation and common commands.
- Release checklist is documented.
- Existing unit, snapshot, and CLI integration tests pass.

Result:

- `--version` and `--help` are exposed through clap metadata.
- `docs/Release.md` records local install, release verification, release
  contents, and versioning notes.
- CLI integration tests cover `--version` and `--help`.

### Milestone 20

VSCode Language Association

Status: Done

Make `*.html.erb` files work predictably in VSCode.

Context:

Shopify Ruby / Ruby LSP appears to associate `erb`, but may not associate
`html.erb` by default. erbfmt will eventually need either workspace guidance or
a VSCode extension contribution for a `html.erb` language identifier.

Target work:

- Decide the language id to use for `*.html.erb`.
- Document a workspace-level file association fallback.
- Decide whether the first VSCode integration should be settings-only or an
  extension scaffold.
- Keep format-on-save behavior compatible with the current CLI.

Acceptance:

- Roadmap and README explain the recommended VSCode file association.
- Repository workspace settings keep formatting `.html.erb` files on save.
- Future extension requirements are listed before implementation begins.

Result:

- Workspace settings associate `*.html.erb` with the `erb` language id.
- Recommended extensions include Shopify Ruby LSP.
- `docs/VSCode.md` documents language association, format-on-save wiring, and
  future extension requirements.

### Milestone 21

Diagnostic Quality Pass

Status: Done

Improve lint and parse diagnostics so CLI output is easier to act on.

Target work:

- Add line/column information to lexer and parser errors.
- Include file-scoped diagnostics in a stable format.
- Keep messages concise enough for editor task output.
- Add CLI integration tests for diagnostic output where useful.

Acceptance:

- Syntax errors identify at least file, line, and column.
- Existing lint rules still pass.
- Multi-file output remains deterministic.

Result:

- Lexer errors report line and column.
- ERB parser errors report line and column when called through spanned tokens.
- CLI integration tests cover lexer and parser diagnostic locations.

### Milestone 22

Formatter Behavior Audit

Status: Done

Run another focused pass on real-world ERB patterns before wrappers.

Candidate fixtures:

- HTML attributes spanning multiple lines.
- ERB output mixed with text in more inline contexts.
- HTML comments around ERB blocks.
- `begin` / `rescue` / `ensure` shape decisions.
- Forms and Rails helper-heavy templates.

Acceptance:

- Any newly supported behavior is covered by formatter snapshots.
- Unsupported patterns are documented as constraints or future milestones.

Result:

- Added `samples/formatter-audit.html.erb` with Rails-like helper blocks,
  long HTML attributes, inline ERB output, comments, and `begin` / `rescue` /
  `ensure`.
- Added formatter snapshot coverage for the audit fixture.
- Added support for output ERB do-blocks, preserving `<%=` on blocks such as
  `<%= form_with ... do |form| %>`.
- Added `rescue` and `ensure` as ERB branch markers.

### Milestone 23

Pre-release Distribution Decision

Status: Done

Choose the first thin wrapper or integration target now that the Rust CLI has
configuration, diagnostics, and formatter stability coverage.

Target work:

- Decide whether npm, Ruby gem, or VSCode extension comes first.
- Document the wrapper boundary: Rust binary remains the formatter engine.
- Define local verification commands for the chosen wrapper.
- Keep unsupported Ruby semantic analysis out of scope.

Acceptance:

- Roadmap documents the first distribution target and why.
- The chosen wrapper has a minimal implementation plan before code begins.
- Existing Rust tests remain the release gate.

Result:

- Chose VSCode extension as the first thin wrapper because formatter-on-save is
  the highest-leverage local integration.
- Added `editors/vscode` with a JavaScript extension that registers a document
  formatter and invokes the Rust `erbfmt` command.
- Added settings for `erbfmt.command`, `erbfmt.arguments`, and
  `erbfmt.configPath`.
- Kept formatting logic in the Rust binary.

### Milestone 24

VSCode Extension Packaging Pass

Status: Next

Prepare the extension scaffold for real local installation.

Target work:

- Add extension-host tests or a documented manual verification path.
- Decide whether to package with `vsce` now or defer until binary distribution
  is clearer.
- Add local install/debug instructions for the extension.
- Decide how the extension should discover or bundle the `erbfmt` binary.

Acceptance:

- The extension can be launched or packaged locally with documented commands.
- Formatter behavior is verified through the extension path.
- Rust tests remain the formatter release gate.

## Later

Potential future directions:

- npm package
- Ruby gem
- Tree-sitter integration
- Biome integration

These are lower priority than making the Rust formatter reliable and easy to
install for real `*.html.erb` templates.
