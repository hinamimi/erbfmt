# erbfmt Ruby wrapper

This gem is a thin launcher for the platform-specific erbfmt Rust binary.

Initial releases are distributed as platform-specific `.gem` files attached to
the [erbfmt GitHub Release](https://github.com/hinamimi/erbfmt/releases/tag/v0.1.5).
Newer releases may also be published to RubyGems.org. To install a downloaded
release gem directly, choose the file matching the local platform. For example,
on glibc Linux x64:

```bash
gem install --local ./erbfmt-0.1.5-x86_64-linux-gnu.gem
erbfmt --version
```

## Installing from a Gemfile

If the erbfmt version you want is available on RubyGems.org, use Bundler:

```bash
bundle add erbfmt --group development --require false
bundle exec erbfmt --version
```

If the version is only available as a GitHub Release asset, download the
matching release asset, unpack it into `vendor/gems`, and reference the
unpacked path dependency:

```bash
curl -L \
  -o erbfmt-0.1.5-x86_64-linux-gnu.gem \
  https://github.com/hinamimi/erbfmt/releases/download/v0.1.5/erbfmt-0.1.5-x86_64-linux-gnu.gem
mkdir -p vendor/gems
gem unpack erbfmt-0.1.5-x86_64-linux-gnu.gem --target vendor/gems
```

Add the unpacked gem to the project Gemfile:

```ruby
group :development do
  gem "erbfmt",
    path: "vendor/gems/erbfmt-0.1.5-x86_64-linux-gnu",
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
ruby scripts/version.rb set 0.1.5
ruby scripts/version.rb verify 0.1.5
```

Build, install, and execute a local platform-specific gem in isolation:

```bash
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile gem:verify
```

RubyGems.org publishing is an explicit release step handled from the repository
root. See [Release.md](../../docs/Release.md#rubygemsorg-publishing).
