# Configuration

erbfmt reads `erbfmt.json` from the current directory or a parent directory.
You can also pass an explicit path:

```bash
erbfmt --config path/to/erbfmt.json app/views/users/show.html.erb
```

The shape is intentionally close to Biome's `formatter` and `linter`
configuration sections, with small erbfmt-specific `formatter.noHtmlIndent`
and `formatter.indentHtml` options for the existing HTML indentation behavior.

## Example

```json
{
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentWidth": 2,
    "indentHtml": true,
    "noHtmlIndent": false,
    "lineEnding": "lf",
    "lineWidth": 80,
    "trailingNewline": true
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
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
- `formatter.indentHtml`: indent nested HTML tags. This matches the CLI
  behavior controlled by `--no-html-indent`.
- `formatter.noHtmlIndent`: disable nested HTML tag indentation. This is the
  config equivalent of CLI `--no-html-indent`.
- `formatter.lineEnding`: `lf` or `crlf`.
- `formatter.lineWidth`: accepted for compatibility with common formatter
  configs. It is reserved for future wrapping behavior.
- `formatter.trailingNewline`: keep or remove the final newline.

CLI `--no-html-indent` overrides `formatter.indentHtml` and
`formatter.noHtmlIndent`. If both `formatter.indentHtml` and
`formatter.noHtmlIndent` are set, `formatter.noHtmlIndent` wins.

## Linter

- `linter.enabled`: enable or disable lint diagnostics.
- `linter.rules.recommended`: default state for lint rules when a rule is not
  configured explicitly.
- `linter.rules.emptyErbControlBlock`: `error`, `warn`, or `off`.
- `linter.rules.unsupportedErbBlockStarter`: `error`, `warn`, or `off`.

Currently `warn` and `error` both enable the rule. Diagnostic severity is not
reported yet.
