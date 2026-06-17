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
- VSCode diagnostics wrapper for `erbfmt --lint`
- VSCode extension local VSIX packaging
- VSCode `html-erb` syntax highlighting
- Line/column diagnostics for lexer and ERB parser errors
- `erbfmt.json` formatter and linter configuration
- `formatter.lineWidth` wrapping for long HTML and standalone ERB tags
- VSCode ERB-safe `Ctrl+/` comment toggling
- Focused formatter edge-case fixture coverage
- Formatter idempotency tests for current samples
- Binary distribution strategy documentation

Reference samples:

- `samples/sample.html.erb`: intentionally unformatted formatter demo
- `samples/lint-next.html.erb`: lint rule sample
- `samples/stability.html.erb`: formatting stability sample
- `samples/formatter-audit.html.erb`: real-template formatter audit sample
- `samples/formatter-edge-cases.html.erb`: focused formatter edge-case sample

Known constraints:

- Ruby code is not parsed as Ruby AST.
- Formatting currently normalizes most non-meaningful whitespace.
- Distribution wrappers are not published yet.
- The VSCode extension is not published yet.
- The VSCode extension does not bundle the Rust binary yet.

## Immediate Focus

The Rust binary and the thin VSCode wrapper are usable for local pre-release
development. Configuration, syntax diagnostics, syntax highlighting, local VSIX
packaging, ERB-safe comment toggling, focused edge-case coverage, sample
idempotency tests, and binary distribution strategy are in place. The next
milestone should prepare binary release automation before adding another
wrapper.

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
- Added `editors/vscode` with a TypeScript extension that registers a document
  formatter and invokes the Rust `erbfmt` command.
- Added diagnostics wiring that invokes `erbfmt --lint` on open and save.
- Added settings for `erbfmt.command`, `erbfmt.arguments`, and
  `erbfmt.configPath`.
- Added `erbfmt.lint.enabled` for turning diagnostics off.
- Kept formatting logic in the Rust binary.

### Milestone 24

VSCode Extension Packaging Pass

Status: Done

Prepare the extension scaffold for real local installation.

Target work:

- Add extension-host tests beyond the current smoke test.
- Decide whether to package with `vsce` now or defer until binary distribution
  is clearer.
- Add local install/debug instructions for the extension.
- Decide how the extension should discover or bundle the `erbfmt` binary.

Acceptance:

- The extension can be launched or packaged locally with documented commands.
- Formatter behavior is verified through the extension path.
- Diagnostics behavior is verified through the extension path.
- Rust tests remain the formatter release gate.

Result:

- Added TypeScript compile output packaging with `@vscode/vsce`.
- Added `npm run package --prefix editors/vscode` for local VSIX generation.
- Added `.vscodeignore` so generated `out/` is packaged while source, tests,
  and dev-only config are excluded.
- Documented local VSIX build and install commands.
- Kept Rust binary bundling out of scope; local packages still expect an
  installed or configured `erbfmt` binary.

### Milestone 25

Editor Diagnostics Span Pass

Status: Done

Improve lint diagnostics so VSCode can place rule findings on useful ranges
instead of the start of the file.

Target work:

- Add source spans to lint diagnostics for empty ERB blocks.
- Add source spans to unsupported ERB block starter diagnostics.
- Preserve existing CLI output format or document any intentional changes.
- Keep extension diagnostics as a thin consumer of CLI output.

Acceptance:

- VSCode diagnostics for current lint rules point at the relevant ERB tag.
- CLI integration tests cover diagnostic locations where applicable.
- Existing formatter and extension checks pass.

Result:

- Added source locations to lint rule diagnostics while preserving the existing
  CLI diagnostic text shape.
- Empty ERB control block diagnostics now point at the opening ERB tag.
- Unsupported ERB block starter diagnostics now point at the offending ERB tag.
- CLI integration tests cover lint rule line/column output.

### Milestone 26

VSCode Extension Host Test Pass

Status: Done

Move beyond smoke tests by exercising the extension through a real VSCode
extension-host test harness.

Target work:

- Add extension-host tests for document formatting.
- Add extension-host tests for diagnostics ranges.
- Keep the extension as a thin wrapper around the Rust binary.
- Document any setup needed for local extension-host tests.

Acceptance:

- Extension-host tests verify formatting edits through VSCode APIs.
- Extension-host tests verify diagnostics point at the expected ERB tag range.
- Existing Rust and extension smoke checks continue to pass.

Result:

- Added `@vscode/test-electron` and a `npm run test:host` command for local
  extension-host verification.
- Added an extension-host formatting test that invokes VSCode's document
  formatting provider.
- Added an extension-host diagnostics test that checks the lint range on the
  offending ERB tag.
- Documented extension-host test setup in the VSCode extension docs.
- The local sandbox can download the VSCode test build, but Electron exits with
  code 1 in this headless environment; run the host test in a GUI-capable or
  xvfb-backed environment.

### Milestone 27

VSCode Configuration UX Pass

Status: Done

Make local extension failures easier to understand before publishing.

Target work:

- Improve error messages when the configured `erbfmt` command is missing or not
  executable.
- Add a command or diagnostic note that explains which binary path is being
  used.
- Decide whether the extension should offer a bundled-binary path later.
- Keep the current extension package thin until binary distribution is decided.

Acceptance:

- Common setup failures point users toward `cargo build`, `cargo install`, or
  `erbfmt.command`.
- Existing formatter, diagnostics, and extension-host tests pass.

Result:

- Added `erbfmt: Show Command` to display the resolved command line, cwd, and
  config path for the active document.
- Improved formatter and diagnostics failure output for `ENOENT` and `EACCES`
  with setup hints for `cargo build`, `cargo install`, and `erbfmt.command`.
- Fixed lint diagnostics so spawn failures without stderr still surface as a
  visible VSCode diagnostic.
- Kept binary bundling out of scope; the extension remains a thin wrapper.

### Milestone 28

VSCode Publish Metadata Pass

Status: Done

Prepare the local VSIX for eventual marketplace or GitHub release publishing.

Target work:

- Add repository metadata once the canonical repository URL is decided.
- Add categories, keywords, and icon assets suitable for a formatter extension.
- Decide whether the package README should be English-only or include links to
  Japanese docs.
- Keep generated VSIX contents small and predictable.

Acceptance:

- `vsce package --no-dependencies` runs without metadata warnings or only with
  documented intentional warnings.
- Package contents remain limited to runtime files.
- README files explain install and local binary expectations clearly.

Result:

- Added VSCode package metadata for icon, categories, keywords, and gallery
  banner.
- Added a small runtime icon asset at `editors/vscode/media/icon.png`.
- Initially kept `repository` metadata unset until the canonical repository URL
  was known. Milestone 35 later added the GitHub repository metadata.
- Verified VSIX contents remain limited to runtime files.
- Avoided marketplace-specific README links until repository metadata could
  make packaged views stable.

### Milestone 29

VSCode ERB Comment Toggle Pass

Status: Done

Make `Ctrl+/` useful in `*.html.erb` files without accidentally executing ERB
inside HTML comments.

Target work:

- Add a VSCode command for ERB-aware comment toggling.
- Bind `Ctrl+/` / `Cmd+/` for `erb` and `html-erb` documents.
- Toggle ERB tags with ERB comments and HTML fragments with HTML comments.
- Keep mixed HTML/ERB lines safe by splitting comments around ERB tags.

Acceptance:

- ERB control tags toggle between `<% ... %>` and `<%# ... %>`.
- ERB output tags toggle between `<%= ... %>` and `<%#= ... %>`.
- HTML-only lines can be toggled and untoggled.
- Mixed HTML/ERB lines do not leave executable ERB inside HTML comments.

Result:

- Added `erbfmt.toggleComment` and keybindings for `erb` and `html-erb`.
- Added a pure comment transformation module with Node tests.
- Documented the comment behavior in VSCode docs and extension README files.

### Milestone 30

Release Surface Audit

Status: Done

Review the current CLI and VSCode wrapper as a pre-release surface before
adding another wrapper.

Target work:

- Re-run the full verification command set and document any known local-only
  failures.
- Audit README, README_ja, docs, samples, and tasks for stale commands.
- Confirm `erbfmt.json` schema and examples match the implemented config.
- Decide whether the next milestone should improve formatting behavior or start
  npm/Ruby wrapper planning.

Acceptance:

- Docs and sample commands agree with the current binary and extension behavior.
- Known limitations are explicit and not scattered across old milestones.
- The next roadmap direction is chosen from current implementation evidence.

Result:

- Re-ran Rust and VSCode wrapper verification, including local VSIX packaging.
- Documented the intentional `samples/sample.html.erb` unformatted demo role.
- Updated README, README_ja, VSCode, and release docs for current extension
  behavior.
- Kept the missing VSCode `repository` metadata warning as an intentional
  pre-publication limitation.
- Chose formatter correctness on real template edge cases as the next focus.

### Milestone 31

Formatter Edge Case Pass

Status: Done

Improve formatting correctness before adding npm or Ruby distribution wrappers.

Target work:

- Audit real-template patterns that still format awkwardly or too aggressively.
- Add fixtures for inline text mixed with ERB output, multi-line attributes,
  HTML comments around ERB blocks, and helper-heavy Rails templates.
- Tighten formatter behavior only where snapshots clearly describe the intended
  output.
- Keep Ruby semantic parsing out of scope.

Acceptance:

- New or changed formatter behavior is covered by snapshots.
- Unsupported patterns are documented as constraints instead of silently
  reshaped.
- Existing CLI, lint, and VSCode wrapper checks continue to pass.

Result:

- Added `samples/formatter-edge-cases.html.erb` for focused formatter edge
  cases.
- Added snapshot coverage for multi-line HTML attributes, HTML comments around
  ERB blocks, inline text mixed with ERB output, and helper-heavy ERB output.
- Fixed inline HTML elements so existing multi-line opening tags are normalized
  through the same tag formatter instead of being concatenated raw.
- Preserved Ruby expressions inside ERB tags without Ruby semantic parsing.

### Milestone 32

Formatter Idempotency Pass

Status: Done

Make formatter stability explicit across current samples before distribution
wrapper planning.

Target work:

- Add tests that formatting already-formatted sample output is stable.
- Cover `sample`, `stability`, `formatter-audit`, and `formatter-edge-cases`
  fixtures.
- Document any intentional non-idempotent behavior as a bug or constraint.
- Keep wrapper work deferred until sample stability is explicit.

Acceptance:

- Current formatter fixtures are covered by idempotency tests.
- Existing CLI, lint, and VSCode wrapper checks continue to pass.
- The next roadmap direction is chosen from formatter stability evidence.

Result:

- Added formatter idempotency tests for `sample`, `stability`,
  `formatter-audit`, and `formatter-edge-cases` fixtures.
- Confirmed current formatter sample outputs are stable when formatted again.
- Chose binary distribution planning as the next step before npm or Ruby
  wrapper implementation.

### Milestone 33

Binary Distribution Strategy Pass

Status: Done

Decide how local users and editor wrappers should obtain the Rust `erbfmt`
binary before implementing another wrapper.

Target work:

- Compare local `cargo install`, prebuilt binaries, npm wrapper download, Ruby
  gem wrapper download, and VSCode bundled/downloaded binary options.
- Keep the Rust binary as the only formatter engine.
- Decide which distribution path should come first.
- Document development-version behavior for unpublished binaries.

Acceptance:

- Roadmap documents the first binary distribution path and why.
- Docs explain what remains local-only before public releases.
- No wrapper implementation starts until the binary boundary is clear.

Result:

- Chose prebuilt Rust CLI binaries attached to releases as the first public
  distribution target.
- Kept local development on `cargo build` and `cargo install --path .`.
- Deferred npm, Ruby gem, and VSCode binary download/bundling work until
  shared release binaries exist.
- Added `docs/Distribution.md` to compare local install, release binaries, npm,
  Ruby gem, and VSCode binary handling.
- Documented that `0.0.0-dev` should not be used for public binary releases.

### Milestone 34

Binary Release Automation Prep

Status: Done

Prepare the repository for producing prebuilt CLI binaries without publishing
them yet.

Target work:

- Decide the initial release platform matrix.
- Decide archive names and checksum files.
- Add or document a local release build command for the Rust binary.
- Keep publishing and wrapper download logic out of scope.

Acceptance:

- Release docs describe the binary artifact names and platform matrix.
- Local release build verification is documented or scripted.
- Existing CLI, lint, formatter, and VSCode wrapper checks continue to pass.

Result:

- Decided the initial release matrix:
  `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`,
  `aarch64-apple-darwin`, and `x86_64-pc-windows-msvc`.
- Documented archive names and sibling `.sha256` checksum files.
- Added `scripts/package-binary.sh` for local host or explicit-target binary
  archive creation.
- Added a manual-only `Release Binaries` workflow that builds and uploads
  artifacts without publishing a GitHub Release.

### Milestone 35

Repository Publication Prep

Status: Done

Prepare repository metadata for the first GitHub push.

Target work:

- Decide and document the canonical GitHub repository URL.
- Add repository metadata to Rust and VSCode package manifests once the URL is
  known.
- Check README front matter, license, CI, and release docs for public
  repository readiness.
- Keep package publishing out of scope.

Acceptance:

- Repository metadata is consistent across manifests and docs.
- VSCode packaging no longer has intentional missing-repository warnings, or
  the remaining warning is explicitly deferred.
- Existing CI and release-prep checks continue to pass.

Result:

- Chose `https://github.com/hinamimi/erbfmt` as the canonical repository URL
  from the configured `origin` remote.
- Added repository metadata to `Cargo.toml` and the VSCode extension manifest.
- Updated VSCode, release, and distribution docs so missing repository metadata
  is no longer treated as an intentional warning.
- Kept package publishing out of scope.

### Milestone 36

First GitHub Push Verification

Status: Next

Verify the freshly published repository after the first push.

Target work:

- Confirm GitHub Actions run on `main`.
- Inspect CI failures, if any, and fix repository-specific issues.
- Confirm README, license, workflows, and docs render correctly on GitHub.
- Keep package publishing and release publication out of scope.

Acceptance:

- GitHub CI status is known.
- Any first-push CI issues are documented or fixed.
- Repository landing page is readable enough for early contributors.

## Later

Potential future directions:

- npm package
- Ruby gem
- Tree-sitter integration
- Biome integration

These are lower priority than making the Rust formatter reliable and easy to
install for real `*.html.erb` templates.
