# Configuration

erbfmt reads `erbfmt.json` from the current directory or a parent directory.
Create a default config file in the current directory:

```bash
erbfmt init
erbfmt init --force
```

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
      "noDeprecatedHtmlTag": "error",
      "noDuplicateHtmlAttribute": "error",
      "noInvalidHtmlBooleanAttribute": "error",
      "noInvalidHtmlNesting": "error",
      "noSelfClosingHtmlTag": "error",
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
  width are expanded by moving `<%` / `<%=` and `%>` onto their own lines.
  When the Ruby code is a simple command-style method call with top-level
  comma-separated arguments, erbfmt may fold it by adding explicit parentheses
  and placing one argument per line. Ruby expressions that cannot be recognized
  safely are left intact.
- `formatter.trailingNewline`: keep or remove the final newline.

## Linter

- `linter.enabled`: enable or disable lint diagnostics.
- `linter.rules.recommended`: default state for lint rules when a rule is not
  configured explicitly.
- `linter.rules.emptyErbBranch`: `error`, `warn`, or `off`.
- `linter.rules.emptyErbCodeTag`: `error`, `warn`, or `off`.
- `linter.rules.emptyErbControlBlock`: `error`, `warn`, or `off`.
- `linter.rules.noDeprecatedHtmlTag`: `error`, `warn`, or `off`.
- `linter.rules.noDuplicateHtmlAttribute`: `error`, `warn`, or `off`.
- `linter.rules.noInvalidHtmlBooleanAttribute`: `error`, `warn`, or `off`.
- `linter.rules.noInvalidHtmlNesting`: `error`, `warn`, or `off`.
- `linter.rules.noSelfClosingHtmlTag`: `error`, `warn`, or `off`.
- `linter.rules.unsupportedErbBlockStarter`: `error`, `warn`, or `off`.

`error` diagnostics make `erbfmt --lint` exit with a failure status. `warn`
diagnostics are reported, but warning-only lint results exit successfully.
VSCode diagnostics use the matching warning or error severity.

See [LintRules.md](LintRules.md) for the current lint rule design and the next
planned rules.

See [Ignore.md](Ignore.md) for `erbfmt-ignore` lint directives.
