# erbfmt

[日本語](README_ja.md)

A formatter and linter for `*.html.erb` templates, built as a fast Rust CLI.
It formats HTML and ERB control-flow together while preserving Ruby code when
it cannot be changed safely.

> erbfmt is currently in pre-release development. The CLI is usable from the
> repository, but public release binaries, the RubyGems package, and the VSCode
> Marketplace extension are not published yet.

## Install

The current installation method requires a Rust toolchain:

```bash
cargo install --git https://github.com/hinamimi/erbfmt --locked
```

Confirm that the command is available:

```bash
erbfmt --version
erbfmt --help
```

Prebuilt Linux, macOS, and Windows binaries are planned for the first public
release. Platform-specific Ruby gems and the VSCode extension are already
tested as artifacts, but are not published to package registries yet.

## Quick Start

Create `erbfmt.json` in your Rails project:

```bash
cd your-rails-project
erbfmt init
```

Format a file to stdout:

```bash
erbfmt app/views/users/show.html.erb
```

Write formatted output back to files:

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

## Formatting Example

Input:

```erb
<div>
<% if user %>
<p>Hello, <%= user.name %></p>
<% else %>
<p>Please sign in.</p>
<% end %>
</div>
```

Output:

```erb
<div>
  <% if user %>
    <p>Hello, <%= user.name %></p>
  <% else %>
    <p>Please sign in.</p>
  <% end %>
</div>
```

By default, erbfmt indents both HTML nesting and ERB control-flow. It also
formats branches such as `elsif`, `else`, `when`, `rescue`, and `ensure`, and
recognizes output do-blocks such as `<%= form_with ... do |form| %>`.

Long HTML tags are expanded one attribute per line. Simple standalone Ruby
command calls may be wrapped with explicit parentheses when erbfmt can split
their arguments safely. Complex or ambiguous Ruby expressions are preserved.

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
- Expressions that cannot be recognized safely are preserved rather than
  aggressively rewritten.
- Preformatted content such as `pre`, `textarea`, `script`, and `style` is kept
  on the safe side.
- Package registry and Marketplace installation are not available before the
  first public release.

## Documentation

- [Configuration](docs/Configuration.md)
- [Lint Rules](docs/LintRules.md)
- [Ignore Directives](docs/Ignore.md)
- [VSCode Integration](docs/VSCode.md)
- [Development](docs/Development.md)
- [Roadmap](docs/Roadmap.md)
