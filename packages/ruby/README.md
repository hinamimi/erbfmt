# erbfmt Ruby wrapper

This gem is a thin launcher for the platform-specific erbfmt Rust binary.

Initial releases are distributed as platform-specific `.gem` files attached to
the [erbfmt GitHub Release](https://github.com/hinamimi/erbfmt/releases/tag/v0.1.4)
rather than through RubyGems.org. Download the file matching the local platform
and install it directly. For example, on glibc Linux x64:

```bash
gem install --local ./erbfmt-0.1.4-x86_64-linux-gnu.gem
erbfmt --version
```

## Installing from a Gemfile

Since the gem is not on RubyGems.org, Bundler cannot resolve it from a normal
`source` entry. Download the matching release asset, unpack it into
`vendor/gems`, and reference the unpacked path dependency:

```bash
curl -L \
  -o erbfmt-0.1.4-x86_64-linux-gnu.gem \
  https://github.com/hinamimi/erbfmt/releases/download/v0.1.4/erbfmt-0.1.4-x86_64-linux-gnu.gem
mkdir -p vendor/gems
gem unpack erbfmt-0.1.4-x86_64-linux-gnu.gem --target vendor/gems
```

Add the unpacked gem to the project Gemfile:

```ruby
group :development do
  gem "erbfmt",
    path: "vendor/gems/erbfmt-0.1.4-x86_64-linux-gnu",
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
ruby scripts/version.rb set 0.1.4
ruby scripts/version.rb verify 0.1.4
```

Build, install, and execute a local platform-specific gem in isolation:

```bash
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile gem:verify
```

The gem is not published to RubyGems.org.
