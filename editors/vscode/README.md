# erbfmt VSCode Extension

Japanese documentation is included in `README_ja.md`.

Thin VSCode wrapper for the Rust `erbfmt` binary.

## Behavior

- contributes a `html-erb` language id for `*.html.erb`.
- contributes syntax highlighting for HTML plus ERB tags.
- registers a document formatter for `erb` and `html-erb`.
- runs `erbfmt --lint` on open and save to publish diagnostics.
- calls the configured `erbfmt` command and replaces the document with stdout.
- provides `erbfmt: Show Command` to inspect the resolved command, cwd, and
  config path.
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

Use `erbfmt: Show Command` from the command palette to inspect which command,
working directory, and config path the extension resolved for the active
document.

Alternatively, install the Rust binary first:

```bash
cargo install --path ../..
```

Use `erbfmt.configPath` to force a specific `erbfmt.json`; otherwise the
extension searches from the formatted file toward the filesystem root.

Set `erbfmt.lint.enabled` to `false` to disable diagnostics.

If formatting or diagnostics fail with `ENOENT` or `EACCES`, run `cargo build`,
install `erbfmt`, or set `erbfmt.command` to an executable absolute path.

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

Run extension-host tests when you need to verify the wrapper through VSCode
APIs. This command builds the Rust binary first and may download a test VSCode
build on first run. It needs an environment where VSCode/Electron can launch;
headless Linux environments may need `xvfb-run` or an equivalent display setup.

From the repository root:

```bash
npm run test:host --prefix editors/vscode
```

From `editors/vscode`:

```bash
npm run test:host
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
