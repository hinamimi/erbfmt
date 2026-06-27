# Distribution Strategy

erbfmt keeps the Rust binary as the only formatter engine. npm packages, Ruby
gems, and editor extensions should stay thin wrappers around that binary.

## Decision

The first public distribution consists entirely of assets attached to a release
in `https://github.com/hinamimi/erbfmt`: prebuilt `erbfmt` binaries,
platform-specific gems, checksums, and a thin VSIX.

Local development remains:

```bash
cargo build
cargo install --path .
```

Every wrapper delegates to the Rust binary. The gem packages one matching
binary; the VSIX expects a separately installed or configured binary.

## Options

### Local `cargo install`

Status: current path.

Pros:

- Simple and already documented.
- Good for contributors and early testers.
- Avoids premature release automation.

Cons:

- Requires a Rust toolchain.
- Awkward for editor-only users.

Use this for local development until public binaries exist.

### Prebuilt Release Binaries

Status: first public target; the four-platform workflow has been rehearsed.

Pros:

- Keeps Rust as the canonical engine.
- Gives VSCode, npm, and Ruby wrappers one shared binary source.
- Avoids requiring Rust for normal users.

Cons:

- Requires release automation and platform decisions.

Initial target platforms:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Archive names:

- `erbfmt-${version}-x86_64-unknown-linux-gnu.tar.gz`
- `erbfmt-${version}-x86_64-apple-darwin.tar.gz`
- `erbfmt-${version}-aarch64-apple-darwin.tar.gz`
- `erbfmt-${version}-x86_64-pc-windows-msvc.zip`

Each archive should have a sibling `.sha256` file. Public releases should use
the same artifact names that the manual release-binary workflow produces.

### npm Wrapper

Status: deferred beyond `0.1.0`.

The npm package should expose the `erbfmt` CLI and resolve a platform-specific
binary. It should not reimplement formatting logic in TypeScript.

### Ruby Gem Wrapper

Status: implemented and verified for the first public release.

The Ruby gem should expose the `erbfmt` CLI for Ruby/Rails projects and resolve
a platform-specific binary. It should not parse Ruby or ERB separately from the
Rust binary.

The initial wrapper uses same-name platform-specific gems with a Ruby launcher
and one packaged Rust binary. It does not build Rust or download binaries during
gem installation. The four variants are attached to the GitHub Release after
they are built and verified from the release tag. See [RubyGem.md](RubyGem.md)
for the complete design.

### VSCode Binary Handling

Status: publish a thin VSIX with `0.1.0`; defer bundling or download logic.

The VSCode extension currently expects an installed or configured binary. Once
prebuilt release binaries exist, the extension can either:

- keep using `erbfmt.command` and document installation, or
- download/cache a matching binary from release assets.

The `0.1.0` VSIX is attached to the GitHub Release and is not published to the
VSCode Marketplace. It expects `erbfmt` on `PATH` or an explicit
`erbfmt.command`. Bundling large binaries directly into the VSIX should be
avoided until package size and platform strategy are clear.

### Registry Policy

Current pre-1.0 releases use GitHub Release assets only. Standalone binaries,
checksums, platform-specific gems, and the VSIX are downloaded from the same
release.

Do not publish current pre-1.0 releases to RubyGems.org, crates.io, npm,
GitHub Packages, the VSCode Marketplace, or Open VSX. Registry publication can
be reconsidered after the release process and artifact formats have remained
stable for a while.

## Release Version

Public release versions are updated from the repository root:

```bash
ruby scripts/version.rb set <version>
ruby scripts/version.rb verify <version>
```

Before publishing artifacts:

- confirm the canonical repository URL,
- build binaries, gems, and the VSIX from the same Git revision,
- verify `erbfmt --version` against the release version,
- keep registry publication decisions separate from GitHub Release uploads.

See [Release.md](Release.md) for the concrete release procedure.
