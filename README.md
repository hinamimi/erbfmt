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
- Token abstraction
- Lexer scaffold

### Planned

- ERB lexer
- AST parser
- Formatter
- Snapshot testing
- VSCode integration

## Example

### Input

```erb
<div>
<% if user %>
<p>Hello</p>
<% end %>
</div>
```

### Output

```erb
<div>
  <% if user %>
    <p>Hello</p>
  <% end %>
</div>
```

## Development

```bash
cargo run samples/sample.erb
cargo test
```
