# erbfmt VSCode Extension

Thin VSCode wrapper for the Rust `erbfmt` binary.

## Behavior

- contributes a `html-erb` language id for `*.html.erb`.
- registers a document formatter for `erb` and `html-erb`.
- calls the configured `erbfmt` command and replaces the document with stdout.
- keeps formatting logic in the Rust binary.

## Local Development

Install the Rust binary first:

```bash
cargo install --path ../..
```

Or point the extension at the local checkout:

```json
{
  "erbfmt.command": "cargo",
  "erbfmt.arguments": ["run", "--quiet", "--"]
}
```

Use `erbfmt.configPath` to force a specific `erbfmt.json`; otherwise the
extension searches from the formatted file toward the filesystem root.
