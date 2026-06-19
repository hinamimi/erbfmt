# erbfmt Ruby wrapper

This gem is a thin launcher for the platform-specific erbfmt Rust binary.

```ruby
group :development do
  gem "erbfmt", require: false
end
```

```bash
bundle exec erbfmt app/views/users/show.html.erb
```

The wrapper does not implement formatting or linting in Ruby.

## Development

From the repository root:

```bash
cargo build --locked
BUNDLE_GEMFILE=packages/ruby/Gemfile bundle install
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile test verify_version
```

Build, install, and execute a local platform-specific gem in isolation:

```bash
BUNDLE_GEMFILE=packages/ruby/Gemfile \
  ERBFMT_BINARY="$PWD/target/debug/erbfmt" \
  bundle exec rake -f packages/ruby/Rakefile gem:verify
```

The gem is not published yet.
