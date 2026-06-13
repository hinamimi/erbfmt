# AGENTS

This file provides instructions for coding agents working on this repository.

## Project Goal

Build a formatter and linter for ERB templates.

The desired user experience is similar to:

- Prettier
- Biome
- dprint

for HTML/TSX.

## Architecture Principles

### 1. Rust First

The core implementation must remain Rust.

Future wrappers may include:

- npm package
- Ruby gem
- VSCode extension

But the formatter engine should stay in Rust.

### 2. Incremental Development

Prefer small incremental changes.

Do NOT implement parser and formatter simultaneously.

Recommended order:

1. Lexer
2. Parser
3. Formatter
4. Linter

### 3. Keep Dependencies Minimal

Avoid introducing large dependencies unless they provide clear value.

Tree-sitter should not be introduced before the lexer architecture is stable.

### 4. Testing

Prefer automated tests.

Formatter behavior should eventually be covered by snapshot tests.

### 5. MVP Scope

Initially support:

```erb
<% if %>
<% unless %>
<% case %>
<% do %>
<% begin %>
<% end %>
```

Only after these work should additional ERB constructs be added.

#### Non-Goals

At this stage:

- No Ruby AST parsing
- No Rails semantic analysis
- No language server
- No Biome integration

Focus on a working formatter first.
