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

1. Run `cargo build` once.
2. Open the repository root in VSCode.
3. Open the Run and Debug view.
4. Choose `Run erbfmt VSCode Extension`.
5. Press F5.
6. In the new Extension Development Host window, open
   `samples/sample.html.erb`.
7. Run `erbfmt: Format Document`.

`samples/sample.html.erb` is intentionally not formatted. If the extension is
working, running `erbfmt: Format Document` should change its indentation.
VSCode's built-in `Format Document` should also work when erbfmt is the selected
default formatter. If it does not, run `Format Document With...` and choose
`erbfmt`.

The repository workspace settings point the extension at the local Rust
checkout:

```json
{
  "erbfmt.command": "cargo",
  "erbfmt.arguments": ["run", "--quiet", "--"]
}
```

When the extension runs from this checkout, it uses `target/debug/erbfmt` if the
binary exists. If it does not exist yet, it falls back to `cargo run --quiet --`.

`erbfmt.command` must be the executable only. Put extra command-line arguments
in `erbfmt.arguments`.

Alternatively, install the Rust binary first:

```bash
cargo install --path ../..
```

Use `erbfmt.configPath` to force a specific `erbfmt.json`; otherwise the
extension searches from the formatted file toward the filesystem root.

Set `erbfmt.lint.enabled` to `false` to disable diagnostics.
