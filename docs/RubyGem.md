# Ruby Gem Wrapper

This document defines the first Ruby gem wrapper for erbfmt. The Rust binary
remains the only formatter and linter engine.

## Current Status

The wrapper is implemented in `packages/ruby`. It can run a Rust binary through
`ERBFMT_BINARY`, build a platform-specific gem, inspect its platform and binary
metadata, install it into an isolated `GEM_HOME`, and verify `erbfmt --version`.

The manual `Release Binaries` workflow builds this gem on each matching native
runner and uploads it beside the standalone archive. Initial releases attached
those exact verified artifacts to GitHub Releases without publishing them to a
package registry. The v0.1.5 release is the planned point for also publishing
the verified platform gems to RubyGems.org.

## Decision

Start with platform-specific binary gems. Do not provide a source-build gem or
a generic gem that downloads a binary during installation.

Each published gem has:

- the name `erbfmt`;
- the same public version as the Rust crate and GitHub Release;
- a Ruby executable at `exe/erbfmt`; and
- one prebuilt Rust binary at `libexec/erbfmt-bin` or
  `libexec/erbfmt-bin.exe`.

RubyGems executables are Ruby scripts, so `exe/erbfmt` is a small launcher. It
resolves the packaged binary and replaces itself with it using `Kernel.exec`.
This preserves stdin, stdout, stderr, signals, and the Rust process exit status.

There is no Ruby formatter API and no Ruby implementation of ERB parsing.

## Why Prebuilt Gems First

Platform-specific gems provide the intended user experience:

```bash
gem install --local ./erbfmt-0.1.4-x86_64-linux-gnu.gem
erbfmt --version
```

Users do not need a Rust toolchain. Once the matching `.gem` file has been
downloaded from GitHub Releases, installation can be local and offline. After
erbfmt is published to RubyGems.org, the Gemfile workflow can become the normal
`bundle add erbfmt --group development --require false` flow.

A source-build fallback would require Rust and duplicate the concerns already
handled by the release-binary workflow. A generic gem with install-time download
logic would add checksum, proxy, offline, and cache behavior to the wrapper.
Both are deferred.

## Initial Platforms

Build the following gem variants from the matching Rust release binaries:

| RubyGems platform | Rust target | Packaged binary |
| --- | --- | --- |
| `x86_64-linux-gnu` | `x86_64-unknown-linux-gnu` | `erbfmt-bin` |
| `x86_64-darwin` | `x86_64-apple-darwin` | `erbfmt-bin` |
| `arm64-darwin` | `aarch64-apple-darwin` | `erbfmt-bin` |
| `x64-mingw-ucrt` | `x86_64-pc-windows-msvc` | `erbfmt-bin.exe` |

The release workflow verifies these names with current RubyGems and Bundler.
Windows uses RubyInstaller UCRT even though the standalone Rust executable uses
the MSVC target.

Linux starts with glibc only. Alpine/musl and Linux arm64 require additional
Rust release targets and separate gem variants.

Do not publish a generic `ruby` platform variant initially. Unsupported
platforms should fail dependency resolution clearly instead of installing a gem
that cannot run.

## Repository Layout

Keep the wrapper isolated from the Rust crate:

```text
packages/ruby/
  Gemfile
  Rakefile
  erbfmt.gemspec
  exe/erbfmt
  lib/erbfmt.rb
  lib/erbfmt/binary.rb
  lib/erbfmt/version.rb
  libexec/
  test/binary_test.rb
  test/integration_test.rb
  test/test_helper.rb
```

Prebuilt binaries are staging artifacts and are not committed. The gem build
task copies one verified release binary into `libexec`, builds the
platform-specific gem, and removes the staged copy afterward.

The gemspec uses an explicit file list that includes the staged binary. Do not
derive `spec.files` only from `git ls-files`, because the binary is intentionally
untracked.

New platform gems also include `erbfmt.gemspec` itself. This makes `gem unpack`
produce a Bundler-readable path gem without asking users to run
`gem spec --ruby` manually. When that unpacked gemspec is evaluated from a
directory named `erbfmt-<version>-<platform>`, it infers the same platform from
the directory name.

## Launcher Resolution

`Erbfmt::Binary.path` resolves in this order:

1. `ERBFMT_BINARY`, for repository development and tests only.
2. The binary packaged next to the gem under `libexec`.

The launcher must not search `PATH` for `erbfmt`; doing so can recurse into the
RubyGems launcher itself. If the binary is absent or not executable, print one
actionable error to stderr and exit nonzero.

The executable remains a transparent CLI boundary:

```ruby
exec(Erbfmt::Binary.path, *ARGV)
```

Configuration discovery, formatting, linting, and exit codes remain owned by
the Rust binary.

## Gemspec Policy

Initial metadata:

- `name`: `erbfmt`
- `license`: `MIT`
- `required_ruby_version`: `>= 3.1`
- `bindir`: `exe`
- `executables`: `erbfmt`
- no runtime gem dependencies
- source, changelog, and issue tracker metadata point to
  `https://github.com/hinamimi/erbfmt`
- `spec.platform` is set explicitly by the platform build task

The repository uses Ruby 3.4 for wrapper development, but the launcher should
remain compatible with Ruby 3.1 and newer.

## Versioning

Public gem versions exactly match the Rust crate, CLI output, tag, and GitHub
Release version. A gem must contain the binary built from the same tagged commit.

The unpublished development wrapper used RubyGems version `0.0.0.dev` while
Cargo and the VSCode extension used `0.0.0-dev`. The current release version
`0.1.4` is identical everywhere.

`lib/erbfmt/version.rb` is the gem version source. The release verification task
must compare its normalized value with `Cargo.toml` and `erbfmt --version`.

Release gems are produced by the workflow described in [Release.md](Release.md).
Attach them only after every platform variant passes installation and execution
tests from the tagged commit.

## Bundler And Ruby LSP

### Installing from a Gemfile

#### RubyGems.org

After the platform gems are published to RubyGems.org, Bundler can resolve the
matching local platform directly from the normal gem source:

```bash
bundle add erbfmt --group development --require false
bundle exec erbfmt --version
```

Or write the dependency manually:

```ruby
group :development do
  gem "erbfmt", require: false
end
```

Bundler should select the platform gem that matches the current RubyGems
platform. Unsupported platforms, including Alpine/musl and Linux arm64, still do
not currently have a gem.

#### GitHub Release Fallback

The initial GitHub-only distribution does not provide a RubyGems package index.
Bundler therefore cannot resolve erbfmt from a normal `source` entry alone. The
most reliable Gemfile setup is to unpack the platform-specific release gem into
the project and reference it as a path gem.

For glibc Linux x64, first download the matching GitHub Release asset:

```bash
curl -L \
  -o erbfmt-0.1.4-x86_64-linux-gnu.gem \
  https://github.com/hinamimi/erbfmt/releases/download/v0.1.4/erbfmt-0.1.4-x86_64-linux-gnu.gem
```

Then unpack it into `vendor/gems`. New erbfmt gems include
`erbfmt.gemspec`, so Bundler can read the unpacked directory directly as a path
gem.

```bash
mkdir -p vendor/gems
gem unpack erbfmt-0.1.4-x86_64-linux-gnu.gem --target vendor/gems
```

Add the unpacked gem as a path dependency without auto-requiring Ruby code:

```ruby
group :development do
  gem "erbfmt",
    path: "vendor/gems/erbfmt-0.1.4-x86_64-linux-gnu",
    require: false
end
```

Install and run it through the project bundle:

```bash
bundle install
bundle exec erbfmt app/views/users/show.html.erb
```

Commit the unpacked `vendor/gems/erbfmt-...` directory and `Gemfile.lock` when
the project should be installable by other developers without a separate erbfmt
download step. You may also commit the downloaded `.gem` under `vendor/cache`
as the original release artifact, but the Gemfile path entry reads the unpacked
directory.

Older erbfmt gems that do not include `erbfmt.gemspec` still need the manual
`gem spec <asset>.gem --ruby > vendor/gems/.../erbfmt.gemspec` workaround. New
release gems should not.

The package must match the local RubyGems platform:

| Development platform | Release gem |
| --- | --- |
| glibc Linux x64 | `erbfmt-0.1.4-x86_64-linux-gnu.gem` |
| macOS Intel | `erbfmt-0.1.4-x86_64-darwin.gem` |
| macOS Apple Silicon | `erbfmt-0.1.4-arm64-darwin.gem` |
| Windows RubyInstaller UCRT x64 | `erbfmt-0.1.4-x64-mingw-ucrt.gem` |

Projects used on multiple platforms should unpack every required variant under
`vendor/gems` and choose the path that matches the current platform in the
Gemfile. Unsupported platforms, including Alpine/musl and Linux arm64, do not
currently have a gem.

```ruby
erbfmt_platform = Gem::Platform.local.to_s

group :development do
  gem "erbfmt",
    path: "vendor/gems/erbfmt-0.1.4-#{erbfmt_platform}",
    require: false
end
```

If your local RubyGems platform string differs from the release asset name,
map it explicitly in the Gemfile.

One-off local installation does not need a Gemfile:

```bash
gem install --local ./erbfmt-0.1.4-x86_64-linux-gnu.gem
erbfmt --version
```

Do not use a Git source as a substitute:

```ruby
# Unsupported: the repository does not contain a staged Rust binary.
gem "erbfmt", git: "https://github.com/hinamimi/erbfmt.git", tag: "v0.1.4"
```

The Rust binary is inserted only while each release gem is built. Installing
the gemspec directly from the Git repository would produce a launcher without
the binary it needs.

### Ruby LSP

erbfmt is not a Ruby LSP add-on and does not need to be inserted into Ruby
LSP's composed bundle. It can coexist in the project Gemfile without sharing
formatter implementation or Ruby dependencies with Ruby LSP.

The current erbfmt VSCode extension can use the bundled executable with:

```json
{
  "erbfmt.command": "bundle",
  "erbfmt.arguments": ["exec", "erbfmt"]
}
```

The extension runs from the active document directory, allowing Bundler to find
the project Gemfile in that directory or a parent.

## RubyGems.org Publishing

RubyGems.org publishing remains an explicit maintainer step after GitHub Release
assets are built and verified. Use `scripts/publish-rubygems.sh` with the four
platform gems downloaded from the release.

The script reads the API key from `RUBYGEMS_API_KEY`, `GEM_HOST_API_KEY`,
`API_KEY`, or matching entries in `.env`. It never prints the key. RubyGems
itself reads `GEM_HOST_API_KEY`, so the script exports the chosen key under that
name before running `gem push`.

```bash
mkdir -p release-assets
gh release download v0.1.5 --pattern '*.gem' --dir release-assets

scripts/publish-rubygems.sh \
  --version 0.1.5 \
  --asset-dir release-assets \
  --dry-run

scripts/publish-rubygems.sh \
  --version 0.1.5 \
  --asset-dir release-assets \
  --yes
```

## Build And Test Boundary

The scaffold includes:

- unit tests for packaged path and `ERBFMT_BINARY` override resolution;
- a launcher test that verifies arguments and process exit status;
- `gem build` metadata and file-content checks;
- installation into an isolated `GEM_HOME`;
- `erbfmt --version` execution against the staged Rust binary;
- a version consistency check against `Cargo.toml`; and
- one Linux CI job that builds the Rust binary and verifies the local gem; and
- a four-platform release matrix that verifies each native gem and uploads it
  as a workflow artifact.

The release matrix has successfully built, installed, executed, and uploaded
all four platform-specific gems. A stable `0.1.0` gem can also be rehearsed in
an isolated copy without the development-version activation allowance.

Local development uses:

```bash
cargo build
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" bundle exec ruby \
  packages/ruby/exe/erbfmt --version
```

RubyGems.org credentials must not be committed. Publishing uses the local
environment or `.env` via `scripts/publish-rubygems.sh`, and remains separate
from the automatic tag workflow.

## References

- [RubyGems specification reference](https://guides.rubygems.org/specification-reference)
- [RubyGems gem structure and platforms](https://guides.rubygems.org/what-is-a-gem/)
- [Bundler Gemfile reference](https://bundler.io/v4.0/man/gemfile.5.html)
- [Ruby LSP composed bundle](https://shopify.github.io/ruby-lsp/composed-bundle.html)
