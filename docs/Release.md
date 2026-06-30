# Release Notes

## Binary

The canonical binary name is `erbfmt`.

Ruby gems and editor extensions are thin wrappers around the Rust binary rather
than separate formatter engines. An npm wrapper remains deferred.

See [Distribution.md](Distribution.md) for the binary distribution strategy.
See [RubyGem.md](RubyGem.md) for the Ruby gem wrapper design.

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
ruby scripts/test/version_test.rb
ruby scripts/version.rb verify
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

The release binary platform matrix is:

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

The `Release Binaries` GitHub Actions workflow can run manually for rehearsals
or as a reusable workflow for tagged releases. It uploads binary archives,
matching platform-specific Ruby gems from four native runners, one `ruby`
fallback gem, and the thin VSIX. Each native runner installs its gem into an
isolated `GEM_HOME` and executes `erbfmt --version`.

For an unpublished stable-version rehearsal, provide the optional workflow
input while running from `main`:

```bash
gh workflow run release-binaries.yml \
  --ref main \
  -f rehearsal_version=0.1.0
```

The input runs `ruby scripts/version.rb set` only inside each ephemeral runner.
For an actual release commit or tag, leave the input empty so artifact versions
come directly from the checked-out files.

Ruby gem names should be:

- `erbfmt-${version}.gem`
- `erbfmt-${version}-x86_64-linux-gnu.gem`
- `erbfmt-${version}-x86_64-darwin.gem`
- `erbfmt-${version}-arm64-darwin.gem`
- `erbfmt-${version}-x64-mingw-ucrt.gem`

The VSCode artifact should contain:

- `erbfmt-vscode-${version}.vsix`

The VSIX does not contain a Rust binary. Release notes must direct users to a
standalone binary or the Ruby gem before installing the extension.

Initial releases attach the standalone archives, checksums, Ruby gems, and VSIX
to GitHub Releases. They are not pushed to package registries or extension
marketplaces.

Starting with the v0.1.5 release, the Ruby gems may also be published to
RubyGems.org after the GitHub Release assets have been produced and verified.
Keep RubyGems publishing as an explicit maintainer action; do not let the tag
workflow publish gems automatically until the release process has more history.

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
- `scripts/*.rb`
- `scripts/test/*.rb`
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

The repository is currently set to `0.2.1` for the next prerelease.
Earlier milestone work used the fixed development version `0.0.0-dev`, and the
first public prerelease used `0.1.0`.

The canonical CLI version is read from `Cargo.toml`. Ruby gem and VSCode
versions are checked against it by `scripts/version.rb`.

Before a public release:

- Run `ruby scripts/version.rb set <version>` to update every version source,
  lockfile entry, and VSIX filename example.
- Run `ruby scripts/version.rb verify <version>`.
- Confirm `cargo run --quiet -- --version` prints the new version.
- Confirm `erbfmt --version` after local install.
- Confirm the `Release` workflow produced all four expected archives, sibling
  `.sha256` files, Ruby gems, and the VSIX from the release tag.

## Tagged Release Workflow

The `Release` workflow runs when a tag matching `v*.*.*` is pushed, then
requires the exact `vMAJOR.MINOR.PATCH` form. It:

1. verifies that the tag version matches every repository version source;
2. calls `Release Binaries` to build on Linux, Intel and Apple Silicon macOS,
   and Windows;
3. collects an exact set of 14 binary, checksum, gem, and VSIX assets;
4. verifies every standalone archive checksum; and
5. creates a draft GitHub Release with generated notes.

Pre-1.0 versions are marked as prereleases. The workflow never publishes the
draft automatically. Review the generated notes and assets, smoke test the
matching local platform, and publish it manually.

The workflow refuses to overwrite assets on an already published release. A
rerun may update assets only while the release remains a draft.

## RubyGems.org Publishing

RubyGems.org publishing is a separate step after the GitHub Release workflow has
created and verified the `.gem` assets.

Prerequisites:

- a RubyGems.org API key with permission to push `erbfmt`;
- the key available as `RUBYGEMS_API_KEY`, `GEM_HOST_API_KEY`, or `API_KEY`; or
- a local `.env` file containing one of those names.

The `.env` file must remain untracked. The publish script reads the key but never
prints it. RubyGems itself reads `GEM_HOST_API_KEY`, so the script exports the
chosen key under that name before running `gem push`.

Download the gem assets from the draft or published GitHub Release:

```bash
mkdir -p release-assets
gh release download v0.2.1 \
  --pattern '*.gem' \
  --dir release-assets
```

Validate the asset set without publishing:

```bash
scripts/publish-rubygems.sh \
  --version 0.2.1 \
  --asset-dir release-assets \
  --dry-run
```

Publish the fallback gem and all four platform gems:

```bash
scripts/publish-rubygems.sh \
  --version 0.2.1 \
  --asset-dir release-assets \
  --yes
```

The script expects exactly these RubyGems assets for the version:

- `erbfmt-${version}.gem`
- `erbfmt-${version}-x86_64-linux-gnu.gem`
- `erbfmt-${version}-x86_64-darwin.gem`
- `erbfmt-${version}-arm64-darwin.gem`
- `erbfmt-${version}-x64-mingw-ucrt.gem`

If a release version was already partially published to RubyGems.org before the
fallback gem existed, do not rerun the full publish script for the already
published platform gems. Build or download `erbfmt-${version}.gem` and push only
that missing fallback variant:

```bash
gem push release-assets/erbfmt-${version}.gem --host https://rubygems.org
```

After publishing, verify from a clean Bundler project that RubyGems.org can
resolve and install erbfmt. The local platform should use a matching platform
gem when one exists; the `ruby` fallback gem is present for unsupported
platforms in multi-platform lockfiles.

```bash
bundle add erbfmt --group development --require false
bundle exec erbfmt --version
```

For the next release:

```bash
ruby scripts/version.rb set 0.2.1
ruby scripts/version.rb verify 0.2.1
# Run the release verification commands above, then commit.
git tag -a v0.2.1 -m "erbfmt 0.2.1"
git push origin main
git push origin v0.2.1
```
