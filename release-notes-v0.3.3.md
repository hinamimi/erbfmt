# erbfmt v0.3.3

This release adds local quick fixes to the VS Code extension, so you can fix or
ignore individual diagnostics directly from the editor.

## VS Code quick fixes

Place the cursor on an erbfmt diagnostic and press `Ctrl+.` (`Cmd+.` on macOS):

- **Format this change** applies only the selected formatting difference.
- Apply supported lint fixes individually: double-quote attribute values, correct
  self-closing HTML tags, simplify redundant boolean attributes, or remove empty
  ERB tags.
- **Ignore … on this line** inserts an ERB comment that suppresses only the
  selected lint rule on the next line.
- **Ignore formatting for the next node** preserves the next HTML element or ERB
  node, including its children.

Quick fixes refresh diagnostics after applying a change. Stale diagnostics do
not produce edits. Ignore actions are offered at supported markup boundaries;
parser errors cannot be suppressed.

## CLI integration

- JSON lint diagnostics now include a `rule` field. Diagnostics without a lint
  rule, such as parser errors, use `null`.
- The double-quote assist action's target range now covers the attribute, while
  its edit still replaces only the value. This allows the action to match the
  lint diagnostic at the attribute name.
- Editor edits account for Rust UTF-8 byte offsets and VS Code UTF-16 positions,
  including templates containing Japanese text or emoji.

## Upgrade

Upgrade both the CLI and VS Code extension to v0.3.3. Lint quick fixes require the
new CLI rule IDs; the VSIX does not bundle or download a CLI binary.

For RubyGems installations, after the v0.3.3 gems have been published:

```bash
gem install erbfmt -v 0.3.3
```

For Bundler projects pinned to a previous version, update the Gemfile constraint
to `0.3.3` and run `bundle update erbfmt`.

Alternatively, use the standalone binary archive for your platform from this
release, or install from the tagged source:

```bash
cargo install --git https://github.com/hinamimi/erbfmt --tag v0.3.3 --locked
```

Install `erbfmt-vscode-0.3.3.vsix` with VS Code's **Extensions: Install from
VSIX...** command.

## Release assets

The tagged release workflow builds standalone binaries and SHA-256 checksums for
Linux x64, macOS Intel, macOS Apple Silicon, and Windows x64, plus four platform
Ruby gems, a binary-free Ruby fallback gem, and the VSIX (14 assets total).

As a pre-1.0 release, v0.3.3 remains beta software. Review formatting diffs before
committing and pin the CLI version in automated environments.
