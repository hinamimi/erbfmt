# erbfmt VSCode Extension

Thin VSCode wrapper for the Rust `erbfmt` binary.

## Behavior

- contributes a `html-erb` language id for `*.html.erb`.
- registers a document formatter for `erb` and `html-erb`.
- runs `erbfmt --lint` on open and save to publish diagnostics.
- calls the configured `erbfmt` command and replaces the document with stdout.
- keeps formatting logic in the Rust binary.

## Local Development

From this repository, use VSCode's Extension Development Host:

1. Open the repository root in VSCode.
2. Open the Run and Debug view.
3. Choose `Run erbfmt VSCode Extension`.
4. Press F5.
5. In the new Extension Development Host window, open
   `samples/sample.html.erb`.
6. Run `Format Document`.

The repository workspace settings point the extension at the local Rust
checkout:

```json
{
  "erbfmt.command": "cargo",
  "erbfmt.arguments": ["run", "--quiet", "--"]
}
```

Alternatively, install the Rust binary first:

```bash
cargo install --path ../..
```

Use `erbfmt.configPath` to force a specific `erbfmt.json`; otherwise the
extension searches from the formatted file toward the filesystem root.

Set `erbfmt.lint.enabled` to `false` to disable diagnostics.
