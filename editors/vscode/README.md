# erbfmt VSCode Extension

Thin VSCode wrapper for the Rust `erbfmt` binary.

## Behavior

- contributes a `html-erb` language id for `*.html.erb`.
- contributes syntax highlighting for HTML plus ERB tags.
- registers a document formatter for `erb` and `html-erb`.
- runs `erbfmt --lint` on open and save to publish diagnostics.
- calls the configured `erbfmt` command and replaces the document with stdout.
- keeps formatting logic in the Rust binary.

## Local Development

From this repository, use VSCode's Extension Development Host:

1. Run `cargo build` once.
2. Run `npm install --prefix editors/vscode` once.
3. Open the repository root in VSCode.
4. Open the Run and Debug view.
5. Choose `Run erbfmt VSCode Extension`.
6. Press F5.
7. In the new Extension Development Host window, open
   `samples/sample.html.erb`.
8. Run `erbfmt: Format Document`.

The F5 launch configuration runs `npm run compile --prefix editors/vscode`
before starting the Extension Development Host.

This repository includes `.node-version` for nodenv. The current local Node
version is `24.10.0`.

`samples/sample.html.erb` is intentionally not formatted. If the extension is
working, running `erbfmt: Format Document` should change its indentation.
VSCode's built-in `Format Document` should also work when erbfmt is the selected
default formatter. If it does not, run `Format Document With...` and choose
`erbfmt`.

When the extension runs from this checkout, it uses `target/debug/erbfmt` if the
binary exists. If it does not exist yet, it falls back to `cargo run --quiet --`.
Running `cargo build` first is the most reliable local setup because VSCode may
not be able to spawn `cargo` in every environment.

`erbfmt.command` must be the executable only. Put extra command-line arguments
in `erbfmt.arguments`.

Alternatively, install the Rust binary first:

```bash
cargo install --path ../..
```

Use `erbfmt.configPath` to force a specific `erbfmt.json`; otherwise the
extension searches from the formatted file toward the filesystem root.

Set `erbfmt.lint.enabled` to `false` to disable diagnostics.

## TypeScript

The extension source lives in `src/extension.ts` and compiles to
`out/extension.js`.

From the repository root:

```bash
npm run check --prefix editors/vscode
npm run compile --prefix editors/vscode
npm test --prefix editors/vscode
```

Biome handles formatting and linting for the extension code.

```bash
npm run format --prefix editors/vscode
npm run lint --prefix editors/vscode
```

From `editors/vscode`, omit the `--prefix editors/vscode` part:

```bash
npm run check
npm run compile
npm test
```

## Local Package

Build a local VSIX package:

From the repository root:

```bash
npm run package --prefix editors/vscode
```

From `editors/vscode`:

```bash
npm run package
```

Install the generated VSIX from the repository root:

```bash
code --install-extension editors/vscode/erbfmt-vscode-0.0.0-dev.vsix
```

Or from `editors/vscode`:

```bash
code --install-extension erbfmt-vscode-0.0.0-dev.vsix
```

The packaged extension does not bundle the Rust binary yet. Install `erbfmt`
separately or configure `erbfmt.command` to point at a local binary.
