# VSCode Integration

## Language Association

This repository associates `*.html.erb` files with the extension-provided
`html-erb` language id:

```json
{
  "files.associations": {
    "*.html.erb": "html-erb"
  }
}
```

The `html-erb` language id contributes syntax highlighting for HTML with ERB
tags. Ruby tooling such as Shopify Ruby LSP may still use the `erb` language id;
the extension registers formatter support for both `erb` and `html-erb`.

## First-party Extension Scaffold

The repository includes a thin TypeScript VSCode extension in `editors/vscode`.

The extension:

- contributes a `html-erb` language id for `*.html.erb`.
- contributes TextMate syntax highlighting for HTML plus ERB tags.
- registers a document formatter for both `erb` and `html-erb`.
- publishes diagnostics by invoking `erbfmt --lint` on open and save.
- publishes warning diagnostics for lines that differ from erbfmt output.
- invokes the configured `erbfmt` command and uses stdout as the formatted
  document.
- provides `erbfmt: Show Command` to inspect the resolved command, cwd, and
  config path for the active document.
- provides `Ctrl+/` comment toggling for ERB-safe line comments.
- keeps all formatting behavior in the Rust binary.

## Binary Resolution

The extension resolves the command in this order:

1. If `erbfmt.command` is configured, use that executable and pass
   `erbfmt.arguments` before erbfmt's own arguments.
2. If the active file is inside this checkout and `target/debug/erbfmt` exists,
   use that binary.
3. If the active file is inside this checkout but `target/debug/erbfmt` does not
   exist yet, run `cargo run --quiet --` from the checkout root.
4. Otherwise, use `erbfmt` from `PATH`.

Use `erbfmt: Show Command` from the command palette to inspect the resolution
source, command line, working directory, checkout path, local binary path, and
config path for the active document.

For local development, build the binary first:

```bash
cargo build
```

When the extension runs from this checkout, it uses `target/debug/erbfmt` if the
binary exists. This is more reliable than asking VSCode to spawn `cargo`
directly.

Alternatively, install the binary:

```bash
cargo install --path .
```

The extension searches for `erbfmt.json` from the formatted file upward. Set
`erbfmt.configPath` to force a specific config file.

Set `erbfmt.lint.enabled` to `false` to disable lint diagnostics.
Set `erbfmt.formatDiagnostics.enabled` to `false` to disable warnings for
documents that are not formatted.

Use `erbfmt: Show Command` from the command palette when setup fails. If the
extension reports `ENOENT` or `EACCES`, run `cargo build`, install `erbfmt`, or
set `erbfmt.command` to an executable absolute path.

Future binary download support should use the shared release artifacts described
in [Distribution.md](Distribution.md): resolve the host platform, download the
matching archive to extension global storage, verify the sibling `.sha256`, and
fall back to `erbfmt.command` for users who want a pinned binary.

The extension overrides `Ctrl+/` for `erb` and `html-erb` documents. ERB tags
are toggled as ERB comments, HTML fragments as HTML comments, and mixed
HTML/ERB lines are split so ERB code is not executed inside an HTML comment.

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

Install it locally:

From the repository root:

```bash
code --install-extension editors/vscode/erbfmt-vscode-0.1.4.vsix
```

From `editors/vscode`:

```bash
code --install-extension erbfmt-vscode-0.1.4.vsix
```

The package includes the compiled extension JavaScript in `out/`, but it does
not bundle the Rust `erbfmt` binary yet.

Binary bundling or download logic is deferred until prebuilt release binaries
exist. See [Distribution.md](Distribution.md) for the current strategy.

The package repository metadata points at
`https://github.com/hinamimi/erbfmt`, with `editors/vscode` as the extension
directory.

Run extension-host tests when you need to verify the wrapper through VSCode
APIs:

```bash
npm run test:host --prefix editors/vscode
```

This command builds the Rust binary first and may download a test VSCode build
on first run. It needs an environment where VSCode/Electron can launch;
headless Linux environments may need `xvfb-run` or an equivalent display setup.

## Recommended Extensions

The workspace recommends:

- `Shopify.ruby-lsp` for Ruby and ERB language support.
- `rust-lang.rust-analyzer` for erbfmt development.
- `editorconfig.editorconfig` and `streetsidesoftware.code-spell-checker` for
  editor hygiene.

## Future Extension Requirements

The first-party VSCode extension still needs:

- a binary distribution or download story for users who do not build erbfmt
  locally, based on the shared Rust binary release strategy.
- clearer behavior when Ruby LSP is also installed.
