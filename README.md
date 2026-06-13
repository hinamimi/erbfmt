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
- HTML tag indentation by default
- In-place formatting with `--write`
- VSCode workspace format-on-save setup
- Basic linter with lexer, parser, and HTML balance diagnostics
- Syntax lint rules for empty ERB blocks and unsupported ERB control keywords
- Format checking with `--check`
- File-scoped CLI diagnostics

### Planned

- Multi-file CLI

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

Check whether a file is already formatted:

```bash
cargo run -- --check samples/sample.html.erb
```

`--write`, `--check`, and `--lint` are mutually exclusive. `--no-html-indent`
can be used with formatting and checking, but not with linting.

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
