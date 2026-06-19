# Release Notes

## Binary

The canonical binary name is `erbfmt`.

For now, npm packages, Ruby gems, and editor extensions should be treated as
future wrappers around the Rust binary instead of separate formatter engines.

See [Distribution.md](Distribution.md) for the binary distribution strategy.
See [FirstRelease.md](FirstRelease.md) for the first public release plan.
See [RubyGem.md](RubyGem.md) for the platform-specific gem wrapper design.

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
cargo run --quiet -- samples/real-template-audit.html.erb
cargo run --quiet -- --lint samples/sample.html.erb
cargo run --quiet -- --lint samples/stability.html.erb
npm test --prefix editors/vscode
npm run package --prefix editors/vscode
BUNDLE_GEMFILE=packages/ruby/Gemfile bundle install
cargo build --locked
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile gem:verify
```

Run the intentional failing lint fixture separately:

```bash
cargo run --quiet -- --lint samples/lint-next.html.erb
cargo run --quiet -- --lint samples/html-parse-errors.html.erb
```

These commands are expected to exit with a failure status because
`samples/lint-next.html.erb` intentionally contains lint issues and
`samples/html-parse-errors.html.erb` intentionally contains an HTML close tag
mismatch.

`npm run package --prefix editors/vscode` should package without repository
metadata warnings once the canonical repository URL is set in the VSCode
manifest.

## Binary Artifacts

The first release binary platform matrix is:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Archive names should be:

- `erbfmt-${version}-x86_64-unknown-linux-gnu.tar.gz`
- `erbfmt-${version}-x86_64-apple-darwin.tar.gz`
- `erbfmt-${version}-aarch64-apple-darwin.tar.gz`
- `erbfmt-${version}-x86_64-pc-windows-msvc.zip`

Each archive should have a sibling `.sha256` checksum file.

Build a local archive for the current host platform:

```bash
scripts/package-binary.sh
```

Build a local archive for an explicit installed Rust target:

```bash
scripts/package-binary.sh x86_64-unknown-linux-gnu
```

The `Release Binaries` GitHub Actions workflow is manual-only
(`workflow_dispatch`) and uploads binary archives as workflow artifacts. It does
not publish a GitHub Release yet.

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
- `.github/workflows/*.yml`
- `scripts/*.sh`
- `packages/ruby/**/*.rb`
- `packages/ruby/Gemfile*`
- `packages/ruby/*.gemspec`
- `packages/ruby/Rakefile`
- `packages/ruby/README.md`
- `packages/ruby/LICENSE.txt`
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
- `samples/real-template-audit.html.erb` covers table, turbo-frame, and
  render-heavy real-template formatting.
- `samples/lint-next.html.erb` intentionally contains lint issues and should
  fail `--lint`.

## Versioning

During active development, erbfmt intentionally uses the fixed development
version `0.0.0-dev` for both the Rust crate and the VSCode extension. Do not
bump minor versions for ordinary milestone work while the project is still in
this phase.

The first public release should use `0.1.0`.

The CLI version is read from `Cargo.toml`. The VSCode extension version is read
from `editors/vscode/package.json`.

Before a public release:

- Replace `0.0.0-dev` with the release version in `Cargo.toml` and
  `editors/vscode/package.json`.
- Regenerate the lockfiles if needed.
- Confirm `cargo run --quiet -- --version` prints the new version.
- Confirm `erbfmt --version` after local install.
- Confirm the manual `Release Binaries` workflow produced all four expected
  archives and sibling `.sha256` files from the release tag or tagged commit.
