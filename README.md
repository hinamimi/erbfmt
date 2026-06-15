# erbfmt

A fast formatter and linter for Ruby ERB templates.

## Goals

- Format ERB templates like Prettier formats TSX.
- Preserve Ruby code blocks while formatting surrounding HTML.
- Handle nested ERB control structures correctly.
- Be fast enough for editor-on-save formatting.
- Written in Rust.

## Current Status

Early development.

### Implemented

- CLI scaffold
- File input
- ERB lexer
- Lightweight HTML tokenizer
- HTML-aware mixed parser
- AST parser
- Mixed AST-driven formatter
- ERB block indentation
- ERB branch formatting for `else`, `elsif`, `when`, `rescue`, and `ensure`
- Case block formatting with `when` branches
- Output ERB do-block formatting such as `<%= form_with ... do |form| %>`
- ERB output inside HTML tag attributes
- HTML tag indentation by default
- In-place formatting with `--write`
- VSCode workspace format-on-save setup
- Basic linter with lexer, parser, and HTML balance diagnostics
- Syntax lint rules for empty ERB blocks and remaining unsupported ERB block starters
- Format checking with `--check`
- File-scoped CLI diagnostics
- Line/column diagnostics for syntax and lint findings
- `erbfmt.json` formatter and linter configuration
- Long HTML tag wrapping controlled by `formatter.lineWidth`
- Multi-file lint, check, and write modes
- Thin VSCode extension with formatter, diagnostics, syntax highlighting, and
  ERB-safe comment toggling

## Example

### Input

```erb
<div>
<% if user %>
<ul>
<% Objects.map do |obj| %>
<p>Hello</p>
<% end %>
</ul>
<% elsif guest? %>
<p>Guest</p>
<% else %>
<p>Please sign in</p>
<% end %>
<% case role %>
<% when "admin" %>
<p>Admin</p>
<% when "user" %>
<p>User</p>
<% end %>
</div>
```

### Output

```erb
<div>
  <% if user %>
    <ul>
      <% Objects.map do |obj| %>
        <p>Hello</p>
      <% end %>
    </ul>
  <% elsif guest? %>
    <p>Guest</p>
  <% else %>
    <p>Please sign in</p>
  <% end %>
  <% case role %>
  <% when "admin" %>
    <p>Admin</p>
  <% when "user" %>
    <p>User</p>
  <% end %>
</div>
```

## CLI

Install the local checkout as `erbfmt`:

```bash
cargo install --path .
```

Confirm the installed binary:

```bash
erbfmt --version
erbfmt --help
```

erbfmt reads `erbfmt.json` from the current directory or a parent directory.
Use `--config` to pass a specific file:

```bash
erbfmt --config erbfmt.json samples/sample.html.erb
```

Format a file:

```bash
cargo run -- samples/sample.html.erb
erbfmt samples/sample.html.erb
```

Format a file in place:

```bash
cargo run -- --write samples/sample.html.erb
erbfmt --write samples/sample.html.erb
```

Lint a file:

```bash
cargo run -- --lint samples/sample.html.erb
erbfmt --lint samples/sample.html.erb
```

Check whether a file is already formatted:

```bash
cargo run -- --check samples/sample.html.erb
erbfmt --check samples/sample.html.erb
```

Lint or check multiple files:

```bash
cargo run -- --lint samples/sample.html.erb samples/lint-next.html.erb
cargo run -- --check samples/sample.html.erb samples/lint-next.html.erb
```

`--write`, `--check`, and `--lint` are mutually exclusive.

By default, erbfmt indents both ERB control-flow blocks and HTML tag nesting.
Set `"indentHtml": false` in `erbfmt.json` to keep HTML indentation unchanged
and only indent ERB blocks.

`formatter.lineWidth` controls when long HTML tags are expanded one attribute
per line with the closing marker on its own line.

Long standalone ERB tags also use `formatter.lineWidth`, but erbfmt does not
split Ruby expressions. When a standalone ERB tag is too long, only the ERB tag
markers are expanded:

```erb
<%=
  link_to "Edit profile", edit_user_path(user), class: "button button--primary"
%>
```

## Samples

- `samples/sample.html.erb`: intentionally unformatted formatter demo.
- `samples/stability.html.erb`: fixed stability fixture for formatter output.
- `samples/formatter-audit.html.erb`: Rails-like formatter audit fixture.
- `samples/lint-next.html.erb`: intentionally invalid lint fixture.

## VSCode

This repository includes a thin VSCode extension scaffold in `editors/vscode`.
It registers `erbfmt` as a document formatter for `*.html.erb` files while
keeping the formatter and lint engines in the Rust binary. The extension also
invokes `erbfmt --lint` on open and save to publish diagnostics.

For local extension development, build the binary first:

```bash
cargo build
```

When the extension runs from this checkout, it uses `target/debug/erbfmt` if the
binary exists. You can also point the wrapper at another command:

```json
{
  "erbfmt.command": "/absolute/path/to/erbfmt",
  "erbfmt.arguments": []
}
```

The workspace also includes RunOnSave settings as a fallback.

Install the recommended `emeraldwalk.RunOnSave` extension, then saving a `.html.erb` file runs:

```bash
cargo run --quiet -- --write "${file}"
```

See [docs/VSCode.md](docs/VSCode.md) for extension and workspace integration
notes.

## Development

```bash
cargo fmt
cargo check --all-targets
cargo clippy
cargo test
cargo run -- samples/sample.html.erb
```

See [docs/Release.md](docs/Release.md) for local release verification.
See [docs/Configuration.md](docs/Configuration.md) for formatter and linter configuration.
See [docs/VSCode.md](docs/VSCode.md) for VSCode extension packaging and local
install notes.
