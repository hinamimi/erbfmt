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
cargo run --quiet -- --lint samples/sample.html.erb
cargo run --quiet -- --lint samples/stability.html.erb
cargo run --quiet -- --lint samples/lint-next.html.erb
```

`samples/lint-next.html.erb` is expected to exit with a failure status because
it intentionally contains lint issues.

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
- `docs/*.md`
- `erbfmt.json`

## Versioning

The CLI version is read from `Cargo.toml`.

Before a release:

- Update the crate version in `Cargo.toml`.
- Confirm `cargo run --quiet -- --version` prints the new version.
- Confirm `erbfmt --version` after local install.
