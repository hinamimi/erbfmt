# Distribution Strategy

erbfmt keeps the Rust binary as the only formatter engine. npm packages, Ruby
gems, and editor extensions should stay thin wrappers around that binary.

## Decision

The first public distribution path should be prebuilt `erbfmt` binaries attached
to a release in `https://github.com/hinamimi/erbfmt`.

Local development remains:

```bash
cargo build
cargo install --path .
```

Public wrapper work should wait until release binaries exist. This keeps the
binary boundary clear before npm, Ruby gem, or VSCode download logic is added.

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

Status: first public target.

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

Status: defer until prebuilt binaries exist.

The npm package should expose the `erbfmt` CLI and resolve a platform-specific
binary. It should not reimplement formatting logic in TypeScript.

### Ruby Gem Wrapper

Status: designed; implementation remains deferred until prebuilt release
binaries are publishable.

The Ruby gem should expose the `erbfmt` CLI for Ruby/Rails projects and resolve
a platform-specific binary. It should not parse Ruby or ERB separately from the
Rust binary.

The initial wrapper uses same-name platform-specific gems with a Ruby launcher
and one packaged Rust binary. It does not build Rust or download binaries during
gem installation. See [RubyGem.md](RubyGem.md) for the complete design.

### VSCode Binary Handling

Status: defer bundling or download logic.

The VSCode extension currently expects an installed or configured binary. Once
prebuilt release binaries exist, the extension can either:

- keep using `erbfmt.command` and document installation, or
- download/cache a matching binary from release assets.

Bundling large binaries directly into the VSIX should be avoided until package
size and platform strategy are clear.

## Release Version

The repository is currently set to `0.1.0` for first-release preparation. The
earlier `0.0.0-dev` version must not be used for public binaries.

Before publishing binaries:

- verify all version sources with `ruby scripts/version.rb verify 0.1.0`,
- confirm the canonical repository URL,
- build release binaries from the same Git revision,
- verify `erbfmt --version` against the release version.

See [FirstRelease.md](FirstRelease.md) for the concrete first-release plan.
