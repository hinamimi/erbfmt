# erbfmt for VS Code

[日本語](https://github.com/hinamimi/erbfmt/blob/main/editors/vscode/README_ja.md)

**Format, lint, highlight, and comment ERB and HTML+ERB templates directly in
VS Code.**

```diff
-<div><% if user.admin? %><span>Admin</span><% end %></div>
+<div>
+  <% if user.admin? %>
+    <span>Admin</span>
+  <% end %>
+</div>
```

This extension adds editor integration for the fast Rust-based
[erbfmt](https://github.com/hinamimi/erbfmt) CLI. It is designed for Rails
`*.html.erb` templates and keeps formatting and linting in the CLI, so
command-line, CI, and editor results stay consistent.

> [!IMPORTANT]
> This extension does not bundle or download the `erbfmt` binary yet. Install
> the CLI separately, or configure `erbfmt.command` to run your project-local
> command such as `bundle exec erbfmt`.

> [!WARNING]
> erbfmt is beta software. Review formatting diffs before committing them and
> pin an exact CLI version in automated environments.

## Features

- HTML and ERB syntax highlighting for `*.html.erb` files.
- Document formatting for the `html-erb` and `erb` language ids.
- erbfmt lint diagnostics when a document is opened or saved.
- Format diagnostics that warn when a document differs from erbfmt output.
- ERB-safe `Ctrl+/` / `Cmd+/` comment toggling.
- Automatic `erbfmt.json` discovery from the active file's directory.
- Explicit CLI, arguments, and configuration path settings.
- `erbfmt: Show Command` for inspecting the resolved command and working
  directory.

## Install

Install **erbfmt** from the VS Code Marketplace, then install the erbfmt CLI
using one of the methods below.

### RubyGems / Bundler

For Rails projects, Bundler is usually the most convenient way to pin erbfmt:

```bash
bundle add erbfmt --group development --require false
bundle exec erbfmt --version
```

Configure the extension to use the project-pinned command:

```json
{
  "erbfmt.command": "bundle exec erbfmt"
}
```

For a global local command, install the RubyGem directly:

```bash
gem install erbfmt -v 0.1.5
erbfmt --version
```

The global install works with the default extension setting
`"erbfmt.command": "erbfmt"`. Bundler is preferred for project use because it
pins the formatter version.

### GitHub Release Binary

Download the CLI from the
[v0.1.5 GitHub Release](https://github.com/hinamimi/erbfmt/releases/tag/v0.1.5),
extract it, and place `erbfmt` or `erbfmt.exe` on your `PATH`.

### Cargo

With a Rust toolchain, install the tagged source directly:

```bash
cargo install --git https://github.com/hinamimi/erbfmt --tag v0.1.5 --locked
erbfmt --version
```

## Quick Start

Open a `*.html.erb` file and run **Format Document**. If VS Code asks for a
formatter, select **erbfmt**.

To format automatically on save:

```json
{
  "[html-erb]": {
    "editor.defaultFormatter": "erbfmt.erbfmt-vscode",
    "editor.formatOnSave": true
  }
}
```

Create a configuration file at the root of the Rails project:

```bash
erbfmt init
```

The extension searches for `erbfmt.json` from the active document toward the
filesystem root. Use `erbfmt.configPath` only when a workspace needs an
explicit configuration file.

Diagnostics are enabled by default and update when an ERB document is opened or
saved. Lint diagnostics come from `erbfmt --lint`; format diagnostics warn on
lines that differ from erbfmt output. Use **erbfmt: Lint Document** to refresh
them manually.

## Using Bundler

Projects that install erbfmt as a Ruby gem can run the project-pinned version:

```json
{
  "erbfmt.command": "bundle exec erbfmt"
}
```

The extension splits this command into an executable and arguments without using
a shell, then runs it from the workspace or active document directory. Bundler
can find the project's `Gemfile` in that directory or a parent.

## Commands

| Command | Purpose |
| --- | --- |
| `erbfmt: Format Document` | Format the active ERB document. |
| `erbfmt: Lint Document` | Refresh lint diagnostics. |
| `erbfmt: Show Command` | Show the resolved executable, arguments, cwd, and config. |
| `erbfmt: Toggle Comment` | Toggle ERB-safe comments for the current selection. |

## Settings

| Setting | Default | Purpose |
| --- | --- | --- |
| `erbfmt.command` | `erbfmt` | Command used to run erbfmt, such as `bundle exec erbfmt`. |
| `erbfmt.arguments` | `[]` | Additional arguments inserted after `erbfmt.command`. |
| `erbfmt.configPath` | empty | Optional path to a specific `erbfmt.json`. |
| `erbfmt.lint.enabled` | `true` | Publish diagnostics on open and save. |
| `erbfmt.formatDiagnostics.enabled` | `true` | Warn when the document is not formatted. |

Use `erbfmt.arguments` for extra flags that should always be passed before the
file path. For example, `"erbfmt.command": "bundle exec erbfmt"` can still be
combined with `"erbfmt.arguments": ["--some-flag"]`.

## Comments

`Ctrl+/` or `Cmd+/` toggles comments line by line. ERB tags become ERB comments
such as `<%# if user %>` or `<%#= user.name %>`. HTML fragments become HTML
comments. Mixed HTML/ERB lines are split so ERB code is not accidentally
executed inside an HTML comment.

## Troubleshooting

If formatting or diagnostics fail with `ENOENT` or `EACCES`:

1. Confirm `erbfmt --version` works in a terminal.
2. Run **erbfmt: Show Command** and inspect the executable and working directory.
3. Set `erbfmt.command` to `bundle exec erbfmt` or an executable absolute path
   when VS Code cannot see the same `PATH` as the terminal.
4. Run **erbfmt: Show Command** to confirm how `erbfmt.command` was split.

The extension can coexist with Shopify Ruby LSP. It contributes the
`html-erb` language id and registers formatting for both `html-erb` and `erb`.

## Links

- [erbfmt documentation](https://hinamimi.github.io/erbfmt/)
- [CLI repository](https://github.com/hinamimi/erbfmt)
- [Issues](https://github.com/hinamimi/erbfmt/issues)
- [Release notes](https://github.com/hinamimi/erbfmt/releases)

## Development

From the repository root:

```bash
cargo build
npm install --prefix editors/vscode
npm test --prefix editors/vscode
npm run package --prefix editors/vscode
```

Open the repository in VS Code, choose **Run erbfmt VSCode Extension**, and
press F5 to start an Extension Development Host. See the
[VSCode integration documentation](https://github.com/hinamimi/erbfmt/blob/main/docs/VSCode.md)
for extension-host tests, command resolution, and release packaging details.
