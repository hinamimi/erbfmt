# First Public Release Plan

This document records the first public release plan and its verification steps.

Version `0.1.0` was published as a GitHub pre-release with standalone binaries,
checksums, platform-specific gems, and a VSIX.

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
- Four platform-specific `erbfmt` gems as GitHub Release assets.
- `erbfmt-vscode-0.1.0.vsix` as a GitHub Release asset.
- Release notes that explain the MVP scope and known limitations.

Do not publish yet:

- crates.io package
- npm package
- RubyGems.org package
- GitHub Packages
- VSCode Marketplace extension
- Open VSX extension
- automatic GitHub Release workflow

The VSCode extension remains a VSIX-installed wrapper that expects an
installed/configured Rust binary. Marketplace publishing should wait until the
binary download/cache story is implemented. The platform-specific gems package
the same Rust binaries built from the release tag and are attached to the
GitHub Release only after all variants pass installation and execution
verification.

## Version Bump Files

Update these files for the release commit:

- `Cargo.toml`
- `Cargo.lock`
- `packages/ruby/lib/erbfmt/version.rb`
- `packages/ruby/Gemfile.lock`
- `editors/vscode/package.json`
- `editors/vscode/package-lock.json`
- `docs/VSCode.md`
- `editors/vscode/README.md`
- `editors/vscode/README_ja.md`

Update them together and verify the result:

```bash
ruby scripts/version.rb set 0.1.0
ruby scripts/version.rb verify 0.1.0
```

`README.md` and `README_ja.md` are user-facing rather than version sources.
After the release artifact URLs are verified, replace their pre-release notice
and source-only installation text with the final binary installation steps.

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

3. Run `ruby scripts/version.rb set 0.1.0`.
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
ruby scripts/test/version_test.rb
ruby scripts/version.rb verify 0.1.0
cargo run --quiet -- --version
npm run package --prefix editors/vscode
```

Review both READMEs from a new user's perspective. Their installation commands,
supported platforms, package availability, and pre-release status must match
the artifacts that will actually be published.

Before creating the release commit, the four-platform package flow can be
rehearsed without changing repository files:

```bash
gh workflow run release-binaries.yml \
  --ref main \
  -f rehearsal_version=0.1.0
```

The workflow changes versions only inside each runner. Its artifacts must use
`0.1.0`, while the checked-out branch remains unchanged.

After pushing the tag, run the manual `Release Binaries` workflow from the tag
without `rehearsal_version`. Confirm that all standalone artifacts exist:

- `erbfmt-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `erbfmt-0.1.0-x86_64-apple-darwin.tar.gz`
- `erbfmt-0.1.0-aarch64-apple-darwin.tar.gz`
- `erbfmt-0.1.0-x86_64-pc-windows-msvc.zip`

Each artifact must have a sibling `.sha256` file.

Also confirm that the workflow produced:

- `erbfmt-0.1.0-x86_64-linux-gnu.gem`
- `erbfmt-0.1.0-x86_64-darwin.gem`
- `erbfmt-0.1.0-arm64-darwin.gem`
- `erbfmt-0.1.0-x64-mingw-ucrt.gem`
- `erbfmt-vscode-0.1.0.vsix`

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
- the four platform-specific `.gem` files
- `erbfmt-vscode-0.1.0.vsix`

Release notes should include:

- supported platforms
- install example
- `erbfmt --version` check
- MVP scope
- known limitations:
  - no Ruby AST parsing
  - no Rails semantic analysis
  - VSCode extension is not published to the Marketplace
  - VSCode extension requires a separately installed or configured binary
  - package registries are not used for the initial release

Keep the GitHub Release as a draft until archive names, checksums, VSIX version,
and binary version output all match `0.1.0`.

## Installing A Release Gem

Download the verified gem matching the local platform from the GitHub Release,
then install it as a local package. For example, on glibc Linux x64:

```bash
gem install --local ./erbfmt-0.1.0-x86_64-linux-gnu.gem
erbfmt --version
```

Do not rebuild gems locally for the release. All attached variants must contain
the binaries verified from the tagged commit. Install each artifact in a clean
matching environment and confirm `erbfmt --version` before publishing the draft
GitHub Release.

## After Release

Return `main` to `0.0.0-dev` only if continuing the fixed-development-version
policy. Otherwise choose the next development version explicitly in a separate
versioning milestone.
