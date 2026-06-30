# erbfmt

[Documentation](https://hinamimi.github.io/erbfmt/) · [日本語](README_ja.md)

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

> erbfmt is currently in pre-release development. Version `0.2.1` is available
> through RubyGems.org and GitHub Releases.

## Install

For Rails projects, install erbfmt through Bundler so every developer and CI
job uses the same pinned version:

```bash
bundle add erbfmt --group development --require false
bundle exec erbfmt --version
```

Then run erbfmt through Bundler:

```bash
bundle exec erbfmt --write app/views/users/show.html.erb
```

For a global local command, install the RubyGem directly:

```bash
gem install erbfmt -v 0.2.1
erbfmt --version
```

The global install is convenient for quick trials, but Bundler is preferred for
project use because it pins the formatter version.

Alternative installation methods are still available. Download a standalone
archive from the
[v0.2.1 release](https://github.com/hinamimi/erbfmt/releases/tag/v0.2.1), extract
it, and place `erbfmt` or `erbfmt.exe` on your `PATH`.

- Linux x64: `x86_64-unknown-linux-gnu`
- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows x64: `x86_64-pc-windows-msvc`

With a Rust toolchain, install the tagged source directly from GitHub:

```bash
cargo install --git https://github.com/hinamimi/erbfmt --tag v0.2.1 --locked
```

See [Ruby Gem Wrapper](docs/RubyGem.md#installing-from-a-gemfile) for Bundler,
global gem installation, and platform notes.

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

Format a directory recursively:

```bash
erbfmt --write app/views
```

Check formatting without changing files, for example in CI:

```bash
erbfmt --check app/views
```

Run the linter:

```bash
erbfmt --lint app/views
```

Lint output defaults to a human-readable format with source excerpts. Use
`--lint-format plain` when a script needs the compact legacy line format.

`--write`, `--check`, and `--lint` are mutually exclusive. Check mode returns a
nonzero status when formatting would change a file; lint mode does so when an
error-level diagnostic is found. Directory inputs require one of these modes and
are filtered by `files.includes`; without `files.includes`, recursive discovery
uses `*.html.erb` files.

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
formatting, diagnostics, and ERB-safe comment toggling. Install the extension
from the VS Code Marketplace, then install the erbfmt CLI through Bundler,
RubyGems, or a standalone release binary.

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

## Documentation

- [Documentation site](https://hinamimi.github.io/erbfmt/)
- [Configuration](docs/Configuration.md)
- [Lint Rules](docs/LintRules.md)
- [Ignore Directives](docs/Ignore.md)
- [VSCode Integration](docs/VSCode.md)
- [Development](docs/Development.md)
- [Roadmap](docs/Roadmap.md)
