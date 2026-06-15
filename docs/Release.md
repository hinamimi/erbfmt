# Release Notes

## Binary

The canonical binary name is `erbfmt`.

For now, npm packages, Ruby gems, and editor extensions should be treated as
future wrappers around the Rust binary instead of separate formatter engines.

## Local Install

Install from a checkout:

```bash
cargo install --path .
```

Confirm the installed binary:

```bash
erbfmt --version
erbfmt --help
```

## Release Verification

Run these checks before cutting a release:

```bash
cargo fmt
cargo check --all-targets
cargo test
cargo clippy
cargo run --quiet -- samples/sample.html.erb
cargo run --quiet -- samples/stability.html.erb
cargo run --quiet -- samples/formatter-edge-cases.html.erb
cargo run --quiet -- --lint samples/sample.html.erb
cargo run --quiet -- --lint samples/stability.html.erb
npm test --prefix editors/vscode
npm run package --prefix editors/vscode
```

Run the intentional failing lint fixture separately:

```bash
cargo run --quiet -- --lint samples/lint-next.html.erb
```

This command is expected to exit with a failure status because
`samples/lint-next.html.erb` intentionally contains lint issues.

`npm run package --prefix editors/vscode` may warn that the VSCode extension
manifest has no `repository` field. That warning is intentional until the
canonical public repository URL is decided.

## Release Contents

Keep these files in the release verification surface:

- `Cargo.toml`
- `Cargo.lock`
- `src/**/*.rs`
- `tests/**/*.rs`
- `src/snapshots/*.snap`
- `samples/*.html.erb`
- `README.md`
- `README_ja.md`
- `LICENSE.txt`
- `docs/*.md`
- `erbfmt.json`
- `docs/schema/erbfmt.schema.json`
- `editors/vscode/package.json`
- `editors/vscode/src/**/*.ts`
- `editors/vscode/syntaxes/*.json`
- `editors/vscode/media/*`
- `editors/vscode/README*.md`

## Samples

- `samples/sample.html.erb` is intentionally unformatted so formatter demos and
  VSCode Format Document visibly change the file.
- `samples/stability.html.erb` is a fixed stability fixture.
- `samples/formatter-audit.html.erb` is a Rails-like formatter audit fixture.
- `samples/formatter-edge-cases.html.erb` covers focused formatter edge cases.
- `samples/lint-next.html.erb` intentionally contains lint issues and should
  fail `--lint`.

## Versioning

During active development, erbfmt intentionally uses the fixed development
version `0.0.0-dev` for both the Rust crate and the VSCode extension. Do not
bump minor versions for ordinary milestone work while the project is still in
this phase.

The CLI version is read from `Cargo.toml`. The VSCode extension version is read
from `editors/vscode/package.json`.

Before a public release:

- Replace `0.0.0-dev` with the release version in `Cargo.toml` and
  `editors/vscode/package.json`.
- Regenerate the lockfiles if needed.
- Confirm `cargo run --quiet -- --version` prints the new version.
- Confirm `erbfmt --version` after local install.
