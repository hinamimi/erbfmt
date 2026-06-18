# Configuration

erbfmt reads `erbfmt.json` from the current directory or a parent directory.
You can also pass an explicit path:

```bash
erbfmt --config path/to/erbfmt.json app/views/users/show.html.erb
```

The shape is intentionally close to Biome's `formatter` and `linter`
configuration sections, with a small erbfmt-specific `formatter.indentHtml`
option for the existing HTML indentation behavior.

## Example

```json
{
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentWidth": 2,
    "indentHtml": true,
    "lineEnding": "lf",
    "lineWidth": 80,
    "trailingNewline": true
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "emptyErbBranch": "error",
      "emptyErbCodeTag": "error",
      "emptyErbControlBlock": "error",
      "unsupportedErbBlockStarter": "error"
    }
  }
}
```

## Formatter

- `formatter.enabled`: enable or disable formatting.
- `formatter.indentStyle`: `space` or `tab`.
- `formatter.indentWidth`: number of spaces per indent level when using
  `space`.
- `formatter.indentHtml`: indent nested HTML tags.
- `formatter.lineEnding`: `lf` or `crlf`.
- `formatter.lineWidth`: target line width. Opening, void, and self-closing
  HTML tags that exceed this width are expanded one attribute per line, with
  the closing marker on its own line. Standalone ERB tags that exceed this
  width are expanded by moving only `<%` / `<%=` and `%>` onto their own lines;
  Ruby expressions are not split.
- `formatter.trailingNewline`: keep or remove the final newline.

## Linter

- `linter.enabled`: enable or disable lint diagnostics.
- `linter.rules.recommended`: default state for lint rules when a rule is not
  configured explicitly.
- `linter.rules.emptyErbBranch`: `error`, `warn`, or `off`.
- `linter.rules.emptyErbCodeTag`: `error`, `warn`, or `off`.
- `linter.rules.emptyErbControlBlock`: `error`, `warn`, or `off`.
- `linter.rules.unsupportedErbBlockStarter`: `error`, `warn`, or `off`.

Currently `warn` and `error` both enable the rule. Diagnostic severity is not
reported yet.

See [LintRules.md](LintRules.md) for the current lint rule design and the next
planned rules.
