---
name: erbfmt
description: Format, lint, and safely fix Rails HTML+ERB templates with the erbfmt CLI. Use when editing or reviewing .html.erb views, investigating erbfmt diagnostics, applying mechanical fixes, or adding local ignore directives.
---

# erbfmt

Use the project's erbfmt CLI for HTML+ERB formatting and diagnostics. The skill
supplies instructions, not the binary or a second formatter implementation.

## Choose the project command

Inspect the project's `Gemfile`, lockfile, and `erbfmt.json`. If erbfmt is a
project dependency, use `bundle exec erbfmt`; otherwise use the installed
`erbfmt`. Preserve the project's pinned version and configuration. Run from the
project/config directory, or pass `--config PATH` when needed.

Check `--version` and `--help` before relying on flags or JSON fields. v0.3.4 adds
structured syntax-error locations and editor ignore metadata. If no usable CLI
is installed, explain the prerequisite. Install or change dependencies only when
that is part of the requested work; do not guess formatter output. Installing this skill does not install the
CLI. The examples below use `bundle exec erbfmt`; substitute the selected command.

## Check the requested templates

Limit checks and edits to the user's files or relevant changed views:

```bash
bundle exec erbfmt --lint --lint-format json app/views/users/show.html.erb
bundle exec erbfmt --check app/views/users/show.html.erb
```

Lint JSON is newline-delimited, with one object per file **with findings**.
Empty stdout can mean no findings, disabled lint, or an excluded file. Check the
exit status and stderr as well: error diagnostics exit 1; warning/info-only
results exit 0. Configuration, I/O, and usage failures are not JSON diagnostics.
Use `rule` IDs and structured `location` when available, rather than parsing
message wording. For an older CLI, a missing location is not permission to guess
an edit position.

## Apply changes within the user's scope

When formatting is requested or part of the authorized edit, use `--write` on
the intended paths and review the diff. For a review-only request, report findings
without rewriting files.

```bash
bundle exec erbfmt --write app/views/users/show.html.erb
bundle exec erbfmt --assist --assist-format json app/views/users/show.html.erb
```

Assist can quote HTML attributes, correct self-closing HTML tags, simplify
redundant boolean attribute values, and remove empty ERB tags. Apply only the
relevant candidates. `--assist --write PATH` applies all enabled candidates in
that file, so use it when that full scope is intended. Lint ignores do not disable
assist actions.

For JSON edits, `range.start` and `range.end` are zero-based **UTF-8 byte offsets**
with an exclusive end, referring to the original source. An action's target
`range` can be wider than its `edit.range`. Apply nonoverlapping edits from the
end of the source and regenerate candidates if the source changed. Do not use
byte offsets as JavaScript string indices or editor UTF-16 positions. Locations
are 1-based Unicode scalar columns; CRLF occupies two bytes.

## Preserve template behavior

Keep inline text/ERB whitespace, Ruby expressions, raw-text contents, and
attributes intact outside the intended change. Do not resolve a parser rejection
by enabling fragment/optional-closing-tag tolerance without establishing that
the project intends those structures. Cross-ERB-branch HTML remains unsupported;
formatting must not invent matching tags or move Ruby control flow.

Use ignore directives when the user wants to preserve intentional markup or
suppress a specific finding. Prefer ERB comments, which do not render as HTML:

```erb
<%# erbfmt-ignore lint/noDeprecatedHtmlTag: legacy email markup %>
<center>Legacy</center>

<%# erbfmt-ignore format: preserve upstream widget %>
<div   class="widget">Keep spacing</div>
```

`lint/<rule>` affects the next physical target line; `format` preserves the next
complete HTML element or ERB node, including children. In v0.3.4+, consecutive
ignore directives share that target line, preserving existing suppressions.
Do not insert comments inside multiline attributes, Ruby strings, or raw text.
Parser errors cannot be suppressed. Use CLI-provided `ignore` / `ignoreTargets`
metadata when automating insertion, and tolerate its absence on older CLIs.

After a change, rerun lint and check, inspect the diff, and confirm that a second
format pass makes no further changes when formatting was applied. Summarize
remaining findings, exclusions, or CLI failures instead of masking them with
blanket ignores or project-wide configuration changes.

For detailed configuration and CLI examples, consult the project's existing
`erbfmt.json` and the [official command reference](https://hinamimi.github.io/erbfmt/commands.md).
