# erbfmt Ruby wrapper

This gem is a thin launcher for the platform-specific erbfmt Rust binary.

Initial releases are distributed as platform-specific `.gem` files attached to
the erbfmt GitHub Release rather than through RubyGems.org. Download the file
matching the local platform and install it directly. For example, on glibc
Linux x64:

```bash
gem install --local ./erbfmt-0.1.0-x86_64-linux-gnu.gem
erbfmt --version
```

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
ruby scripts/version.rb set 0.1.0
ruby scripts/version.rb verify 0.1.0
```

Build, install, and execute a local platform-specific gem in isolation:

```bash
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile gem:verify
```

The gem is not published to RubyGems.org.
