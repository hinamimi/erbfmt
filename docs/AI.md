# AI Agent Usage

This document describes the stable command surface intended for AI agents such
as Codex and Claude Code.

The hosted command reference for agents is also distributed as Markdown:

```text
https://hinamimi.github.io/erbfmt/commands.md
```

## Recommended Workflow

After editing Rails ERB templates, run:

```bash
erbfmt --lint --lint-format json app/views
erbfmt --check app/views
```

Use `--write` only when the user explicitly wants formatter changes:

```bash
erbfmt --write app/views
```

Use assist when the user wants safe, mechanical edits such as attribute quote
normalization or HTML self-closing tag fixes:

```bash
erbfmt --assist --assist-format json app/views
erbfmt --assist --write app/views
```

## JSON Lint Output

`--lint-format json` prints one JSON object per file with findings. Files without
findings produce no record. This makes the
output easy to consume as newline-delimited JSON for directory or multi-file
runs.

```bash
erbfmt --lint --lint-format json app/views
```

Example output (additional editor metadata is omitted):

```json
{
  "file": "app/views/users/show.html.erb",
  "summary": {
    "issues": 1,
    "errors": 1,
    "warnings": 0
  },
  "diagnostics": [
    {
      "severity": "error",
      "rule": "noDuplicateHtmlAttribute",
      "message": "duplicate HTML attribute `class`",
      "location": {
        "line": 1,
        "column": 32
      }
    }
  ]
}
```

The command exits with a nonzero status when at least one error-level diagnostic
is found. Warning-only lint results still exit successfully.

## JSON Assist Output

`--assist-format json` prints one JSON object per included file, even when there
are no actions. Example for `<div class='card'></div>` (additional editor metadata
is omitted):

```json
{
  "file": "app/views/users/show.html.erb",
  "summary": {
    "actions": 1
  },
  "actions": [
    {
      "id": "html.useDoubleQuotedAttributes",
      "rule": "noNonDoubleQuotedHtmlAttributeValue",
      "title": "Use double quotes for HTML attribute value",
      "location": {
        "line": 1,
        "column": 6
      },
      "range": {
        "start": 5,
        "end": 17
      },
      "edit": {
        "range": {
          "start": 11,
          "end": 17
        },
        "replacement": "\"card\""
      }
    }
  ]
}
```

Assist actions are suggestions unless `--assist --write` is used.

## Safety Notes

- Do not parse human-readable `pretty` output when JSON is available.
- Do not run `--assist --write` unless the user wants safe assist edits applied.
- Treat `location: null` as a file-level diagnostic.
- Prefer preserving complex Ruby expressions over rewriting them manually.
- Use formatter or lint ignore directives only when the template is
  intentionally outside erbfmt's safe formatting or linting model.

See [JsonInterface.md](JsonInterface.md) for field definitions, UTF-8 ranges,
Unicode columns, error streams, exit statuses, and rule/action mappings. New
editor metadata and structured syntax-error locations in that document are
unreleased additions to the v0.3.3 interface.

See [Ignore.md](Ignore.md) for ignore directives.
