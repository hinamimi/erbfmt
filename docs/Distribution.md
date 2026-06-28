# Distribution Strategy

erbfmt keeps the Rust binary as the only formatter engine. npm packages, Ruby
gems, and editor extensions should stay thin wrappers around that binary.

## Decision

The first public distribution started with assets attached to a GitHub Release.
The default user-facing installation path is now RubyGems.org for the CLI and
the VS Code Marketplace for the editor extension. GitHub Releases continue to
host standalone binaries, checksums, Ruby gem assets, and VSIX files for manual
or offline installation.

Local development remains:

```bash
cargo build
cargo install --path .
```

Every wrapper delegates to the Rust binary. Platform-specific gems package one
matching binary; the Ruby fallback gem packages only the launcher; the VSIX
expects a separately installed or configured binary.

## Options

### RubyGems.org

Status: default user-facing CLI installation path.

Pros:

- Natural for Rails projects.
- Bundler pins erbfmt in `Gemfile.lock`.
- Does not require a Rust toolchain for users on supported platforms.

Cons:

- Requires platform-specific binary gem coverage.
- Unsupported platforms may resolve the fallback gem but still need a binary.

Recommended project install:

```bash
bundle add erbfmt --group development --require false
bundle exec erbfmt --version
```

Global install for quick trials:

```bash
gem install erbfmt -v 0.1.5
erbfmt --version
```

### Local `cargo install`

Status: development path.

Use this for local development or source-based testing:

```bash
cargo install --path .
```

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
and one packaged Rust binary. It also publishes a binary-free `ruby` fallback
gem so Bundler can resolve multi-platform lockfiles that include unsupported
platforms. It does not build Rust or download binaries during gem installation.
The four native variants and fallback gem are attached to the GitHub Release
after they are built and verified from the release tag. See
[RubyGem.md](RubyGem.md) for the complete design.

### VSCode Binary Handling

Status: publish a thin VSIX; defer bundling or download logic.

The VSCode extension currently expects an installed or configured binary. Once
prebuilt release binaries exist, the extension can either:

- keep using `erbfmt.command` and document installation, or
- download/cache a matching binary from release assets.

The extension is published through the VS Code Marketplace and a VSIX is also
attached to GitHub Releases. It expects `erbfmt` on `PATH` or an explicit
`erbfmt.command`, such as `bundle exec erbfmt`. Bundling large binaries directly
into the VSIX should be avoided until package size and platform strategy are
clear.

### Registry Policy

Current pre-1.0 releases use RubyGems.org for the Ruby CLI wrapper and the VS
Code Marketplace for the editor extension. GitHub Releases remain the canonical
place for standalone binaries, checksums, gem assets, and VSIX files. crates.io,
npm, GitHub Packages, and Open VSX remain deferred.

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
