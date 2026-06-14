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
- ERB branch formatting for `else`, `elsif`, and `when`
- Case block formatting with `when` branches
- ERB output inside HTML tag attributes
- HTML tag indentation by default
- In-place formatting with `--write`
- VSCode workspace format-on-save setup
- Basic linter with lexer, parser, and HTML balance diagnostics
- Syntax lint rules for empty ERB blocks and remaining unsupported ERB block starters
- Format checking with `--check`
- File-scoped CLI diagnostics
- Multi-file lint, check, and write modes

### Planned

- Packaging preparation

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

`--write`, `--check`, and `--lint` are mutually exclusive. `--no-html-indent`
can be used with formatting and checking, but not with linting.

By default, erbfmt indents both ERB control-flow blocks and HTML tag nesting.
Use `--no-html-indent` to keep HTML indentation unchanged and only indent ERB blocks:

```bash
cargo run -- --no-html-indent samples/sample.html.erb
erbfmt --no-html-indent samples/sample.html.erb
```

## VSCode

This repository includes workspace settings for format on save.
It also associates `*.html.erb` files with the `erb` language id so Ruby tooling
such as Shopify Ruby LSP can recognize them more reliably.

Install the recommended `emeraldwalk.RunOnSave` extension, then saving a `.html.erb` file runs:

```bash
cargo run --quiet -- --write "${file}"
```

See [docs/VSCode.md](docs/VSCode.md) for the workspace language association and
future extension notes.

## Development

```bash
cargo fmt
cargo check --all-targets
cargo clippy
cargo test
cargo run -- samples/sample.html.erb
```

See [docs/Release.md](docs/Release.md) for local release verification.
