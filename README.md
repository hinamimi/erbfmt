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

### Planned

- Snapshot testing
- VSCode integration

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
cargo run -- samples/sample.erb
```

By default, erbfmt indents both ERB control-flow blocks and HTML tag nesting.
Use `--no-html-indent` to keep HTML indentation unchanged and only indent ERB blocks:

```bash
cargo run -- --no-html-indent samples/sample.erb
```

## Development

```bash
cargo fmt
cargo check --all-targets
cargo clippy
cargo test
cargo run -- samples/sample.erb
```
