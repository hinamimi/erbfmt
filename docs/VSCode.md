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

## First-party Extension Scaffold

The repository includes a thin VSCode extension scaffold in
`editors/vscode`.

The extension:

- contributes a `html-erb` language id for `*.html.erb`.
- registers a document formatter for both `erb` and `html-erb`.
- publishes diagnostics by invoking `erbfmt --lint` on open and save.
- invokes the configured `erbfmt` command and uses stdout as the formatted
  document.
- keeps all formatting behavior in the Rust binary.

For local development, either install the binary:

```bash
cargo install --path .
```

or point the extension at the local checkout:

```json
{
  "erbfmt.command": "cargo",
  "erbfmt.arguments": ["run", "--quiet", "--"]
}
```

The extension searches for `erbfmt.json` from the formatted file upward. Set
`erbfmt.configPath` to force a specific config file.

Set `erbfmt.lint.enabled` to `false` to disable diagnostics.

## Format On Save Fallback

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

The first-party VSCode extension still needs:

- packaging with `vsce`.
- tests inside a real VSCode extension host.
- publish-time metadata and icon assets.
- clearer behavior when Ruby LSP is also installed.
- diagnostics with precise spans for lint rules that do not yet report
  line/column.
- clear behavior when Ruby LSP is also installed.
