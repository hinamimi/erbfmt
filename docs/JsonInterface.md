# JSON Interface

This is the CLI contract for editor integrations and agents. The additions in
this working tree are unreleased: structured syntax-error locations, lint
`ignore` edits, assist `rule` IDs, and `ignoreTargets`. Clients must tolerate
missing optional fields when running an older CLI.

## Streams and exit status

| Invocation / result | stdout | stderr | Exit status |
| --- | --- | --- | --- |
| `--lint --lint-format json`, findings | One JSON object per file with findings | Empty for lint findings, including template syntax errors | `1` if any error; `0` for warning/info only |
| JSON lint, no findings or linter disabled | Empty | Empty | `0` |
| `--assist --assist-format json` | One JSON object per included file, even with zero actions | Empty on success | `0`; suggestions alone do not fail |
| A file excluded by `files.includes` | No lint/assist record for that file | Empty | `0` |
| Invalid config, unreadable file, formatting/parser failure outside lint | No JSON error contract; earlier file records may already exist | Human-readable error | `1` |
| Invalid CLI arguments | No JSON error contract | Usage error | `2` |
| `--check` | No JSON contract | Reports unformatted files or errors | `0` if all included files pass; `1` otherwise |

The JSON flags above describe read-only listing. `--assist --write` instead
prints human-readable applied-action summaries; `--write` formatting prints
human-readable file summaries. Neither write mode returns JSON edits.

JSON output is newline-delimited: each record occupies one physical line. Parse
records independently, without relying on property order. A final exit status
applies to the whole invocation, not the last record. Do not assume that every
input file produces a lint record or that stdout is valid JSON on every failure.
Disk inputs are expanded, deduplicated, and sorted by the CLI; use `file` as the
identity instead of depending on input order.

`--stdin-filepath PATH` reads unsaved UTF-8 source from stdin. The path supplies
file selection and the record's `file`; it need not exist. It cannot be combined
with disk inputs or `--write`. Disk reads and writes also require UTF-8.

## Positions and ranges

- `location.line` and `location.column` are **1-based**. Columns count Unicode
  scalar values, not UTF-8 bytes or UTF-16 code units. A tab counts as one scalar.
- `range.start` / `range.end` are **0-based UTF-8 byte offsets**, with an exclusive
  end. Both refer to the original source passed to that CLI invocation.
- VS Code uses UTF-16 positions. Decode the UTF-8 source prefix before converting
  a byte offset to a document position. Do not pass byte offsets directly to
  `TextDocument.positionAt`.
- CRLF occupies two bytes. Returned replacements are literal text; do not
  normalize line endings while applying an edit.
- `location: null` is permitted for a diagnostic without a known source position.
  Syntax errors now provide structured positions whenever the parser/lexer knows
  them. The `message` does not include the structured location a second time.

## Lint record

| Field | Contract |
| --- | --- |
| `file` | File path string used by the CLI. It may be relative. |
| `summary` | Integer counts: `issues`, `errors`, `warnings`, `infos`. `issues` is the sum of the other three. |
| `diagnostics` | Array of the file's findings after configured severity and ignore filtering. |
| `diagnostics[].rule` | Config rule ID, e.g. `noDuplicateHtmlAttribute`; `null` for syntax errors. |
| `diagnostics[].severity` | `error`, `warning`, or `info`. |
| `diagnostics[].message` | Human-readable description; wording is not an identifier. |
| `diagnostics[].location` | `{ "line": 1, "column": 1 }`, or `null`. |
| `diagnostics[].ignore` | Optional TextEdit to suppress that rule on that line; `null` when insertion is unsupported. |

A TextEdit has this shape:

```json
{
  "range": { "start": 0, "end": 0 },
  "replacement": "<%# erbfmt-ignore lint/noDeprecatedHtmlTag: intentionally preserved %>\n"
}
```

`ignore` is generated from template nodes, not a regular expression over source
lines. No insertion is offered inside multiline attribute continuations, Ruby
strings, raw-text contents, or syntax errors. It suppresses only the selected
rule on the next target line. Existing consecutive directives remain effective;
see [Ignore.md](Ignore.md). The reason is editable by the user.

## Assist record

| Field | Contract |
| --- | --- |
| `file` | File path string used by the CLI. |
| `summary.actions` | Number of mechanical assist actions. Ignore suggestions are not included. |
| `actions` | Array of suggestions, in source range / action ID order. |
| `actions[].id` | Assist action ID. |
| `actions[].rule` | Corresponding lint rule ID. |
| `actions[].title` | Human-readable label; wording is not an identifier. |
| `actions[].location` | Position of the start of the target `range`. |
| `actions[].range` | Source target for associating the action with a diagnostic or cursor. |
| `actions[].edit` | TextEdit that applies the change. It may cover less source than the target `range`. |
| `ignoreTargets` | Optional array of safe standalone template-node boundaries. |
| `ignoreTargets[].range` | Full template node range. |
| `ignoreTargets[].line` | 1-based start line of that node. |
| `ignoreTargets[].format` | TextEdit that preserves that node's formatting, or `null` if it does not end on its own line. |

For `<div class='card'></div>`, the quote action targets `class='card'`
(bytes `[5, 17)`) while its edit replaces only `'card'` (bytes `[11, 17)`).
The corresponding location is line 1, column 6.

| Assist ID | Lint rule |
| --- | --- |
| `html.useDoubleQuotedAttributes` | `noNonDoubleQuotedHtmlAttributeValue` |
| `html.fixSelfClosingTags` | `noSelfClosingHtmlTag` |
| `html.simplifyBooleanAttributes` | `noInvalidHtmlBooleanAttribute` |
| `erb.removeEmptyTags` | `emptyErbCodeTag` |

Other lint rules have no mechanical assist. Match an assist to both the rule ID
and the overlapping target range, not just the containing line.

`--assist` explicitly enables assist execution while respecting action toggles
and assist file selection. Suggestions and `--assist --write` are independent
of lint severity/ignore settings; a lint ignore is not an assist disable flag.
`ignoreTargets` are editor suggestions only: `--assist --write` never inserts
ignore comments. It applies mechanical edits from the end of the original
source, skipping overlapping edits. Individual editor actions must verify the
source version before applying an edit.

## Compatibility and verification

Consumers must ignore unknown object fields and handle absent optional fields.
Treat rule/action IDs, position units, range semantics, stream behavior, and
exit statuses as compatibility-sensitive. Changes to these require explicit
migration notes and CLI/editor tests. Do not depend on message or title wording.

`tests/json_contract.rs` exercises stream records, severities, failures,
exclusions, syntax positions, Unicode/CRLF offsets, and composable ignore edits.
The VS Code Extension Host suite validates applying those edits through editor
APIs, including document-version checks.
