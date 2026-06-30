# erbfmt Ruby wrapper

This gem is a thin launcher for the erbfmt Rust binary.

## Installing from a Gemfile

For Rails projects, use Bundler so the formatter version is pinned for every
developer and CI job:

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

## Global Installation

For a global local command, install erbfmt directly from RubyGems.org:

```bash
gem install erbfmt -v 0.2.1
erbfmt --version
```

The global install is convenient for quick trials, but Bundler is preferred for
project use because it pins the formatter version.

RubyGems.org releases include platform-specific gems with the packaged Rust
binary and may include `erbfmt-0.2.1.gem` as a binary-free Bundler fallback for
multi-platform lockfiles. The fallback resolves dependency installation but
needs either a matching platform gem or `ERBFMT_BINARY` to run.

## GitHub Release Fallback

If you need an offline install or a version only available as a GitHub Release
asset, download the
matching release asset, unpack it into `vendor/gems`, and reference the
unpacked path dependency:

```bash
curl -L \
  -o erbfmt-0.2.1-x86_64-linux-gnu.gem \
  https://github.com/hinamimi/erbfmt/releases/download/v0.2.1/erbfmt-0.2.1-x86_64-linux-gnu.gem
mkdir -p vendor/gems
gem unpack erbfmt-0.2.1-x86_64-linux-gnu.gem --target vendor/gems
```

Add the unpacked gem to the project Gemfile:

```ruby
group :development do
  gem "erbfmt",
    path: "vendor/gems/erbfmt-0.2.1-x86_64-linux-gnu",
    require: false
end
```

Then install and run it through Bundler:

```bash
bundle install
bundle exec erbfmt --version
```

New release gems include the gemspec needed by Bundler. If an older downloaded
asset does not unpack `erbfmt.gemspec`, use the fallback in the Ruby gem docs.

Commit the unpacked `vendor/gems/erbfmt-...` directory and `Gemfile.lock` when
the project should be installable by other team members without a separate
download step. You can also commit the downloaded `.gem` under `vendor/cache`
as the original release artifact. See
[RubyGem.md](../../docs/RubyGem.md#installing-from-a-gemfile) for supported
platforms, older gem fallback steps, and multi-platform guidance.

The wrapper packages one Rust binary and does not implement formatting or
linting in Ruby.

## Development

From the repository root:

```bash
cargo build --locked
BUNDLE_GEMFILE=packages/ruby/Gemfile bundle install
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile test verify_version
```

The shared version check covers Cargo, this gem, the VSCode extension, and
their lockfiles. Release versions are updated from the repository root:

```bash
ruby scripts/version.rb set 0.2.1
ruby scripts/version.rb verify 0.2.1
```

Build, install, and execute a local platform-specific gem in isolation:

```bash
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile gem:verify
```

RubyGems.org publishing is an explicit release step handled from the repository
root. See [Release.md](../../docs/Release.md#rubygemsorg-publishing).
