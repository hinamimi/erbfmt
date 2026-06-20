# Ruby Gem Wrapper

This document defines the first Ruby gem wrapper for erbfmt. The Rust binary
remains the only formatter and linter engine.

## Current Status

The wrapper is implemented in `packages/ruby`. It can run a Rust binary through
`ERBFMT_BINARY`, build a platform-specific gem, inspect its platform and binary
metadata, install it into an isolated `GEM_HOME`, and verify `erbfmt --version`.

The manual `Release Binaries` workflow builds this gem on each matching native
runner and uploads it beside the standalone archive. The gem is not published;
RubyGems.org release automation remains future work.

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
bundle add erbfmt --group development
bundle exec erbfmt --version
```

Users do not need a Rust toolchain, and gem installation does not need network
access beyond the normal RubyGems download.

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

RubyGems uses `0.0.0.dev` for the unpublished development wrapper while Cargo
and the VSCode extension keep `0.0.0-dev`. Release versions such as `0.1.0` are
identical everywhere.

`lib/erbfmt/version.rb` is the gem version source. The release verification task
must compare its normalized value with `Cargo.toml` and `erbfmt --version`.

The first public binary release can remain CLI-only as described in
[FirstRelease.md](FirstRelease.md). Publish the gem only after every platform
variant passes installation and execution tests.

## Bundler And Ruby LSP

Rails projects add the CLI without auto-requiring Ruby code:

```ruby
group :development do
  gem "erbfmt", require: false
end
```

Run it through the project bundle:

```bash
bundle exec erbfmt app/views/users/show.html.erb
```

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

The release matrix still needs a successful post-push manual run before the
cross-platform package boundary is considered fully verified.

Local development uses:

```bash
cargo build
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" bundle exec ruby \
  packages/ruby/exe/erbfmt --version
```

Publishing to RubyGems.org, release credentials, MFA setup, and automated gem
pushes remain outside the scaffold milestone.

## References

- [RubyGems specification reference](https://guides.rubygems.org/specification-reference)
- [RubyGems gem structure and platforms](https://guides.rubygems.org/what-is-a-gem/)
- [Bundler Gemfile reference](https://bundler.io/v4.0/man/gemfile.5.html)
- [Ruby LSP composed bundle](https://shopify.github.io/ruby-lsp/composed-bundle.html)
