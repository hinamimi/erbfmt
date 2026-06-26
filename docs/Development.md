# Development

This document is for contributors working on erbfmt itself. User installation
and CLI usage belong in the repository [README](../README.md).

## Toolchains

- Rust stable with `rustfmt` and `clippy`
- Ruby 3.4 and Bundler for the Ruby wrapper
- Node.js 24 and npm for the VSCode extension

The repository contains `.ruby-version` and `.node-version` for local version
managers.

## Repository Setup

Build the Rust CLI:

```bash
cargo build --locked
```

Install Ruby wrapper development dependencies:

```bash
BUNDLE_GEMFILE=packages/ruby/Gemfile bundle install
```

Install VSCode extension dependencies:

```bash
npm ci --prefix editors/vscode
```

## Verification

Run the Rust checks:

```bash
cargo fmt --check
cargo check --all-targets --locked
cargo test --locked
cargo clippy --locked -- -D warnings
```

Run version and Ruby wrapper checks:

```bash
ruby scripts/test/version_test.rb
ruby scripts/version.rb verify
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile gem:verify
```

Run VSCode extension checks:

```bash
npm test --prefix editors/vscode
```

Common Rust and VSCode commands are also available as workspace tasks in
`.vscode/tasks.json`.

## Architecture

The formatter and linter engine remains in Rust. Ruby gems, npm packages, and
editor extensions are thin wrappers around the Rust binary and must not carry a
second ERB formatter implementation.

The core implementation is developed incrementally:

1. `src/lexer/` separates HTML fragments and ERB tags.
2. `src/html/` tokenizes the HTML portions.
3. `src/mixed_parser/` builds the mixed HTML/ERB structure.
4. `src/formatter/` produces formatted output.
5. `src/linter/` reports configurable diagnostics.

Ruby expressions are still treated conservatively. `src/ruby_format.rs`
recognizes only the narrow command-call shapes that can be wrapped without a
full Ruby AST.

## Test Fixtures

- `samples/sample.html.erb`: intentionally unformatted user-facing demo
- `samples/stability.html.erb`: formatting stability fixture
- `samples/formatter-audit.html.erb`: Rails-like formatter audit
- `samples/formatter-edge-cases.html.erb`: focused formatter edge cases
- `samples/real-template-audit.html.erb`: table, turbo-frame, and render-heavy audit
- `samples/lint-next.html.erb`: intentional lint failures
- `samples/html-parse-errors.html.erb`: intentional HTML parse failures

Formatter behavior is covered by unit, integration, idempotency, and snapshot
tests. Do not reformat intentionally unformatted samples unless the fixture's
purpose requires it.

## VSCode Extension Development

The extension source lives in `editors/vscode`. It is written in TypeScript and
uses Biome for formatting and linting. Build and package it with:

```bash
npm test --prefix editors/vscode
npm run package --prefix editors/vscode
```

See [VSCode.md](VSCode.md) for Extension Development Host, local VSIX, and
binary resolution details.

## Release Work

- [Release.md](Release.md): release verification and version tooling
- [FirstRelease.md](FirstRelease.md): first public release procedure
- [Distribution.md](Distribution.md): binary distribution strategy
- [RubyGem.md](RubyGem.md): platform-specific Ruby wrapper
- [Roadmap.md](Roadmap.md): completed and upcoming milestones
