# VSCode Integration

## Language Association

This repository associates `*.html.erb` files with VSCode's `erb` language id:

```json
{
  "files.associations": {
    "*.html.erb": "erb"
  }
}
```

This is a workspace-level fallback for Ruby tooling such as Shopify Ruby LSP,
which may recognize `erb` but not always `html.erb` by default.

## Format On Save

The workspace uses `emeraldwalk.RunOnSave` to format `.html.erb` files:

```json
{
  "emeraldwalk.runonsave": {
    "commands": [
      {
        "match": "\\.html\\.erb$",
        "cmd": "cargo run --quiet -- --write \"${file}\""
      }
    ]
  }
}
```

## Recommended Extensions

The workspace recommends:

- `emeraldwalk.RunOnSave` for format-on-save wiring.
- `Shopify.ruby-lsp` for Ruby and ERB language support.
- `rust-lang.rust-analyzer` for erbfmt development.
- `editorconfig.editorconfig` and `streetsidesoftware.code-spell-checker` for
  editor hygiene.

## Future Extension Requirements

A first-party VSCode extension should eventually contribute:

- a dedicated `html.erb` language id or a deliberate association strategy.
- file pattern support for `*.html.erb`.
- formatter registration that invokes the `erbfmt` binary.
- settings for binary path and formatter options.
- clear behavior when Ruby LSP is also installed.
