# erbfmt

[日本語](README_ja.md)

**A fast, Prettier/Biome-like formatter and linter for ERB and HTML+ERB.**

```diff
-<div><% if user.admin? %><span>Admin</span><% end %></div>
+<div>
+  <% if user.admin? %>
+    <span>Admin</span>
+  <% end %>
+</div>
```

erbfmt formats HTML structure and ERB control flow together while preserving
Ruby expressions that it cannot safely rewrite. It is built as a Rust CLI for
Rails `*.html.erb` templates and works locally, in CI, and through the
first-party VSCode extension.

> [!WARNING]
> erbfmt is beta software. Formatting output, configuration, lint rules, and
> CLI behavior may change without backward compatibility before the stable release.
> Review formatting diffs before committing them and pin an exact version in
> automated environments.

> erbfmt is currently in pre-release development. Version `0.1.4` is available
> through GitHub Releases. Initial releases are not registered with package
> indexes or extension marketplaces.

## Install

Download the archive for your platform from the
[v0.1.4 release](https://github.com/hinamimi/erbfmt/releases/tag/v0.1.4), extract
it, and place `erbfmt` or `erbfmt.exe` on your `PATH`.

- Linux x64: `x86_64-unknown-linux-gnu`
- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows x64: `x86_64-pc-windows-msvc`

With a Rust toolchain, install the tagged source directly from GitHub:

```bash
cargo install --git https://github.com/hinamimi/erbfmt --tag v0.1.4 --locked
```

Confirm that the command is available:

```bash
erbfmt --version
erbfmt --help
```

The release also provides platform-specific `.gem` files and a VSIX. erbfmt is
not published to crates.io, npm, or RubyGems.org, so registry-based installation
commands are not currently supported.

To manage erbfmt through a Rails project's Gemfile, download the matching
platform gem, unpack it into `vendor/gems`, write its gemspec, and reference it
as a path gem:

```bash
curl -L \
  -o erbfmt-0.1.4-x86_64-linux-gnu.gem \
  https://github.com/hinamimi/erbfmt/releases/download/v0.1.4/erbfmt-0.1.4-x86_64-linux-gnu.gem
mkdir -p vendor/gems
gem unpack erbfmt-0.1.4-x86_64-linux-gnu.gem --target vendor/gems
gem spec erbfmt-0.1.4-x86_64-linux-gnu.gem --ruby \
  > vendor/gems/erbfmt-0.1.4-x86_64-linux-gnu/erbfmt.gemspec
```

```ruby
group :development do
  gem "erbfmt",
    path: "vendor/gems/erbfmt-0.1.4-x86_64-linux-gnu",
    require: false
end
```

```bash
bundle install
bundle exec erbfmt --version
```

Use the gem matching each development platform. See
[Ruby Gem Wrapper](docs/RubyGem.md#installing-from-a-gemfile) for platform
names, why `gem spec --ruby` is needed, and multi-platform projects.

## Quick Start

Create `erbfmt.json` in your Rails project:

```bash
cd your-rails-project
erbfmt init
```

Format a file in place:

```bash
erbfmt --write app/views/users/show.html.erb
```

Format multiple files:

```bash
erbfmt --write app/views/users/show.html.erb app/views/users/edit.html.erb
```

Check formatting without changing files, for example in CI:

```bash
erbfmt --check app/views/users/show.html.erb app/views/users/edit.html.erb
```

Run the linter:

```bash
erbfmt --lint app/views/users/show.html.erb
```

`--write`, `--check`, and `--lint` are mutually exclusive. Check mode returns a
nonzero status when formatting would change a file; lint mode does so when an
error-level diagnostic is found.

## What erbfmt Handles

By default, erbfmt indents both HTML nesting and ERB control-flow. It also
formats branches such as `elsif`, `else`, `when`, `rescue`, and `ensure`, and
recognizes output do-blocks such as `<%= form_with ... do |form| %>`.

Long HTML tags are expanded one attribute per line. Simple standalone Ruby
method calls, with or without explicit parentheses, may be wrapped one argument
per line when erbfmt can split them safely. Complex or ambiguous Ruby
expressions are preserved.

Whitespace-sensitive inline output is handled conservatively. Adjacent inline
HTML, adjacent ERB outputs, and ERB blocks that were originally written on one
line are kept inline, even when that is longer than `formatter.lineWidth`.
Subtrees under `pre`, `textarea`, `script`, `style`, `svg`, `math`, and elements
with `contenteditable` or inline `white-space` styles are preserved on the safe
side. `template` and `noscript` subtrees are also preserved rather than being
rewrapped. Their opening tags may still be normalized or wrapped by attribute
when erbfmt can do so without changing the preserved content.

The linter reports malformed HTML structure, invalid list and table nesting,
deprecated or self-closing HTML tags, duplicate attributes, and unsupported or
empty ERB control-flow constructs.

## Configuration

erbfmt searches for `erbfmt.json` in the current directory and its parents.
Generate the default file with:

```bash
erbfmt init
```

Use a specific configuration file with:

```bash
erbfmt --config path/to/erbfmt.json app/views/users/show.html.erb
```

Common options include indentation style and width, HTML indentation, line
width, line endings, and per-rule lint severity. See
[Configuration](docs/Configuration.md) for the complete format and
[Lint Rules](docs/LintRules.md) for available diagnostics.

`formatter.trailingNewline` defaults to `true`, which is appropriate for normal
template files. If an ERB file is intentionally rendered as an inline partial
inside surrounding text, set it to `false` for that file or project to avoid
adding a final newline to the rendered fragment.

Use `erbfmt-ignore` comments when generated or third-party markup must be left
untouched. See [Ignore Directives](docs/Ignore.md) for the supported syntax.

## VSCode

The first-party extension provides `html-erb` syntax highlighting, document
formatting, diagnostics, and ERB-safe comment toggling. It currently needs a
local VSIX installation and an available `erbfmt` command because it is not yet
published to the Marketplace.

See [VSCode Integration](docs/VSCode.md) for current installation and command
resolution details.

## Current Limitations

- Ruby code is not parsed into a full Ruby AST.
- Rails application semantics are not analyzed.
- Standalone ERB trim, escaped, and raw-output markers such as `<%-`, `-%>`,
  `<%%`, and `<%==` are rejected instead of being rewritten unsafely.
- Expressions that cannot be recognized safely are preserved rather than
  aggressively rewritten.
- Preformatted or format-sensitive content such as `pre`, `textarea`, `script`,
  `style`, `svg`, `math`, `template`, `noscript`, `contenteditable` subtrees,
  and inline `white-space` styles is kept on the safe side.
- Initial releases are distributed only through GitHub Releases, not package
  registries or extension marketplaces.

## Documentation

- [Documentation site](https://hinamimi.github.io/erbfmt/)
- [Configuration](docs/Configuration.md)
- [Lint Rules](docs/LintRules.md)
- [Ignore Directives](docs/Ignore.md)
- [VSCode Integration](docs/VSCode.md)
- [Development](docs/Development.md)
- [Roadmap](docs/Roadmap.md)
