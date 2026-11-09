# erbfmt commands for AI agents

Use this file as the command reference for AI coding agents working on Rails
projects that use erbfmt.

## Detect erbfmt

Prefer the project-pinned Bundler command when a `Gemfile` is present:

```bash
bundle exec erbfmt --version
```

Fallback for globally installed erbfmt:

```bash
erbfmt --version
```

## Initialize configuration

Only run this when the user asks to add erbfmt configuration:

```bash
bundle exec erbfmt init
```

Use `--force` only when the user explicitly wants to overwrite an existing
`erbfmt.json`:

```bash
bundle exec erbfmt init --force
```

## Lint after editing

After editing Rails views, run lint diagnostics with machine-readable output:

```bash
bundle exec erbfmt --lint --lint-format json app/views
```

If the project does not use Bundler for erbfmt:

```bash
erbfmt --lint --lint-format json app/views
```

The JSON output is newline-delimited: one JSON object is printed for each file
with diagnostics. Files without diagnostics produce no output.

## Check formatting

Use check mode to detect formatting changes without modifying files:

```bash
bundle exec erbfmt --check app/views
```

## Inspect safe assist actions

Use assist mode to inspect safe mechanical edits:

```bash
bundle exec erbfmt --assist --assist-format json app/views
```

Assist actions include double-quoting safe HTML attribute values, fixing HTML
self-closing tags, simplifying redundant boolean attributes, and removing empty
ERB tags.

## Format when requested

Only write formatting changes when the user asks for formatting or accepts a
formatter-driven edit:

```bash
bundle exec erbfmt --write app/views
```

For a single file:

```bash
bundle exec erbfmt --write app/views/users/show.html.erb
```

## Apply assist actions when requested

Only apply assist actions when the user asks for these safe mechanical edits:

```bash
bundle exec erbfmt --assist --write app/views
```

## JSON lint schema

`--lint-format json` emits one object per file:

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
      "message": "duplicate HTML attribute `class`",
      "location": {
        "line": 1,
        "column": 32
      }
    }
  ]
}
```

`location` may be `null` for file-level diagnostics.

## JSON assist schema

`--assist-format json` emits one object per file:

```json
{
  "file": "app/views/users/show.html.erb",
  "summary": {
    "actions": 1
  },
  "actions": [
    {
      "id": "html.useDoubleQuotedAttributes",
      "title": "Use double quotes for HTML attribute value",
      "location": {
        "line": 1,
        "column": 12
      },
      "range": {
        "start": 11,
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

## Exit status

- `0`: formatting/check/lint succeeded, or lint found warnings only.
- `1`: formatting check failed, lint found an error, or a file/config/parse
  error occurred.
- `2`: invalid CLI usage.

## Safety rules for agents

- Do not parse `pretty` output when `--lint-format json` is available.
- Do not run `--write` unless formatting changes are requested or approved.
- Do not run `--assist --write` unless safe assist edits are requested or
  approved.
- Preserve complex Ruby expressions when the safe edit is unclear.
- Prefer fixing the source over adding ignore directives.
- Use `erbfmt-ignore` only for intentionally unmanaged or generated markup.
