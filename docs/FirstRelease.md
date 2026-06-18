# First Public Release Plan

This document describes the first public release plan. It is a plan only; do not
publish a release as part of ordinary milestone work.

## Version

Use `0.1.0` for the first public release.

Reasoning:

- `0.0.0-dev` is reserved for active local development.
- `0.1.0` communicates that erbfmt is usable but not stable.
- The project is still pre-1.0, so formatter behavior and configuration can
  still change with normal minor releases.

## Release Scope

Publish:

- GitHub Release `v0.1.0`.
- Rust CLI binary archives for the supported platform matrix.
- Sibling `.sha256` files for every archive.
- Release notes that explain the MVP scope and known limitations.

Do not publish yet:

- crates.io package
- npm package
- Ruby gem
- VSCode Marketplace extension
- automatic GitHub Release workflow

The VSCode extension remains a local or VSIX-installed wrapper that expects an
installed/configured Rust binary. Marketplace publishing should wait until the
binary download/cache story is implemented.

## Version Bump Files

Update these files for the release commit:

- `Cargo.toml`
- `Cargo.lock`
- `editors/vscode/package.json`
- `editors/vscode/package-lock.json`
- `docs/VSCode.md` if the VSIX filename changes in examples
- `editors/vscode/README.md` if the VSIX filename changes in examples
- `editors/vscode/README_ja.md` if the VSIX filename changes in examples

Do not update `README.md` or `README_ja.md` unless user-facing commands change.

## Release Branch And Tag

Use `main` for the first release unless there is active unreleased work that
must stay separate.

Suggested flow:

1. Start from a clean `main`.
2. Create a release commit:

   ```bash
   git switch main
   git pull --ff-only
   ```

3. Replace `0.0.0-dev` with `0.1.0` in the version bump files.
4. Run release verification.
5. Commit the version bump:

   ```bash
   git commit -am "release: 0.1.0"
   ```

6. Tag the exact verified commit:

   ```bash
   git tag -a v0.1.0 -m "erbfmt 0.1.0"
   ```

7. Push the commit and tag:

   ```bash
   git push origin main
   git push origin v0.1.0
   ```

## Required Verification

Run the local release checks from [Release.md](Release.md).

Also confirm:

```bash
cargo run --quiet -- --version
npm run package --prefix editors/vscode
```

After pushing the tag, run the manual `Release Binaries` workflow from the tag
or the tagged commit. Confirm that all four artifacts exist:

- `erbfmt-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `erbfmt-0.1.0-x86_64-apple-darwin.tar.gz`
- `erbfmt-0.1.0-aarch64-apple-darwin.tar.gz`
- `erbfmt-0.1.0-x86_64-pc-windows-msvc.zip`

Each artifact must have a sibling `.sha256` file.

Download the workflow artifacts and verify:

```bash
sha256sum -c erbfmt-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
sha256sum -c erbfmt-0.1.0-x86_64-apple-darwin.tar.gz.sha256
sha256sum -c erbfmt-0.1.0-aarch64-apple-darwin.tar.gz.sha256
sha256sum -c erbfmt-0.1.0-x86_64-pc-windows-msvc.zip.sha256
```

On systems without `sha256sum`, use:

```bash
shasum -a 256 -c <checksum-file>
```

## GitHub Release

Create a draft GitHub Release for `v0.1.0`.

Attach:

- the four binary archives
- the four `.sha256` files

Release notes should include:

- supported platforms
- install example
- `erbfmt --version` check
- MVP scope
- known limitations:
  - no Ruby AST parsing
  - no Rails semantic analysis
  - VSCode extension is not published to the Marketplace
  - npm package and Ruby gem are not published

Publish the GitHub Release only after archive names, checksums, and version
output all match `0.1.0`.

## After Release

Return `main` to `0.0.0-dev` only if continuing the fixed-development-version
policy. Otherwise choose the next development version explicitly in a separate
versioning milestone.
