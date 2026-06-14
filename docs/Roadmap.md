# Roadmap

## Project Direction

erbfmt aims to provide a Prettier/Biome-like formatting experience for
`*.html.erb` templates.

The formatter treats ERB as HTML plus control-flow markers. Ruby expressions are
preserved as text; Ruby semantic analysis remains out of scope for now.

## Current Baseline

Implemented:

- Rust CLI for single-file and multi-file use
- `--write`, `--check`, `--lint`, and `--no-html-indent`
- VSCode workspace format-on-save setup
- ERB lexer
- Lightweight HTML tokenizer
- HTML-aware mixed parser
- Mixed AST-driven formatter
- ERB block formatting for `if`, `unless`, `case`, `do`, `begin`, and `end`
- ERB branch formatting for `else`, `elsif`, and `when`
- Attribute ERB output handling inside HTML tags
- Basic syntax lint rules
- Snapshot tests with `insta`
- CLI integration tests that invoke the compiled binary
- Release checklist and local install documentation
- VSCode workspace language association for `*.html.erb`
- Line/column diagnostics for lexer and ERB parser errors
- `erbfmt.json` formatter and linter configuration

Reference samples:

- `samples/sample.html.erb`: ordinary formatting sample
- `samples/lint-next.html.erb`: lint rule sample
- `samples/stability.html.erb`: formatting stability sample

Known constraints:

- Ruby code is not parsed as Ruby AST.
- Formatting currently normalizes most non-meaningful whitespace.
- Lint rule diagnostics do not yet include line/column spans.
- Distribution wrappers and editor extensions are not implemented yet.

## Immediate Focus

The Rust binary is documented enough for local pre-release use, the workspace
has a basic VSCode association for `*.html.erb`, syntax diagnostics include
source locations, and formatter/linter behavior can be configured through
`erbfmt.json`. The next milestone should audit real-template formatter behavior
before expanding to npm/Ruby wrappers.

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

Status: Next

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

## Later

Potential future directions:

- npm package
- Ruby gem
- VSCode extension with `*.html.erb` language association
- Config file support
- Tree-sitter integration
- Biome integration

These are lower priority than making the Rust formatter reliable and easy to
install for real `*.html.erb` templates.
