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
- AST parser
- Basic formatter
- ERB block indentation
- HTML tag indentation by default
- In-place formatting with `--write`
- VSCode workspace format-on-save setup
- Basic linter with lexer and parser diagnostics

### Planned

- More lint rules

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
  <% end %>
</div>
```

## CLI

Format a file:

```bash
cargo run -- samples/sample.html.erb
```

Format a file in place:

```bash
cargo run -- --write samples/sample.html.erb
```

Lint a file:

```bash
cargo run -- --lint samples/sample.html.erb
```

By default, erbfmt indents both ERB control-flow blocks and HTML tag nesting.
Use `--no-html-indent` to keep HTML indentation unchanged and only indent ERB blocks:

```bash
cargo run -- --no-html-indent samples/sample.html.erb
```

## VSCode

This repository includes workspace settings for format on save.
Install the recommended `emeraldwalk.RunOnSave` extension, then saving a `.html.erb` file runs:

```bash
cargo run --quiet -- --write "${file}"
```

## Development

```bash
cargo fmt
cargo check --all-targets
cargo clippy
cargo test
cargo run -- samples/sample.html.erb
```
