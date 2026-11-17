# erbfmt v0.3.4

This release fixes formatter stability issues, improves local quick fixes in VS
Code, and adds an installable erbfmt workflow for coding agents.

## Formatter fixes

- Fix inline boundary whitespace that changed again on a second format pass.
- Preserve generated Ruby method-chain continuation indentation on later passes.
- Re-audit 990 Forem and OpenStreetMap templates: all accepted templates are now
  idempotent with both default and fragment-tolerant parser settings. Parser
  acceptance and the conservative defaults are unchanged.

## VS Code quick fixes

- Generate safe ignore insertion positions in Rust from real template nodes.
- Compose consecutive ignore directives without displacing existing rule or
  formatting suppressions, and deduplicate identical ignore suggestions.
- Reject actions from a stale menu after the document changes.
- Refresh diagnostics after edits with a short debounce, cancelling superseded
  work. Save, manual lint, and applied fixes refresh immediately.
- Run the Extension Host integration suite in CI with a virtual display.

## CLI / JSON interface

- Syntax-error diagnostics now return structured `location` information when
  known. The plain-text CLI keeps its existing location output.
- JSON lint diagnostics include optional `ignore` edits. JSON assist actions
  include the corresponding `rule`, with safe `ignoreTargets` on the file record.
- Document UTF-8 byte ranges, Unicode scalar columns, stream behavior, exit
  statuses, and lint-to-assist mappings, backed by contract tests.

Consumers should use structured locations rather than extracting positions from
message text, tolerate unknown fields, and handle optional metadata missing from
older CLI versions. Upgrade both the CLI and VSIX for the new quick-fix behavior.

## Coding-agent skill

```bash
npx skills add hinamimi/erbfmt
```

The `erbfmt` skill helps agents use the project's existing CLI and configuration
to inspect, format, lint, and safely fix HTML+ERB views. It does not install the
CLI. To target an agent explicitly:

```bash
npx skills add hinamimi/erbfmt --skill erbfmt --agent codex
```

After this tag is published, a pinned installation is also available:

```bash
npx skills add hinamimi/erbfmt@v0.3.4 --skill erbfmt
```

## Upgrade

After the v0.3.4 gems are published, install with `gem install erbfmt -v 0.3.4`,
or update the project's Gemfile constraint and run `bundle update erbfmt`.
Standalone binaries, checksums, platform gems, the Ruby fallback gem, and
`erbfmt-vscode-0.3.4.vsix` are produced by the tagged release workflow.

As a pre-1.0 release, erbfmt remains beta software. Review formatting diffs before
committing and pin the CLI version in automated environments.
