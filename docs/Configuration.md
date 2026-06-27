# Configuration

erbfmt reads `erbfmt.json` from the current directory or a parent directory.
The file is parsed as JSONC, so line comments, block comments, and trailing
commas are allowed.
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
configuration sections, with a small erbfmt-specific `parser` section and
`formatter.indentHtml` option for ERB and HTML-specific behavior.

## Example

```json
{
  "files": {
    "includes": ["**/*.html.erb", "!vendor/**"]
  },
  "parser": {
    "allowHtmlOptionalClosingTags": false
  },
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
      "noNonDoubleQuotedHtmlAttributeValue": "error",
      "noSelfClosingHtmlTag": "error",
      "unsupportedErbBlockStarter": "error"
    }
  }
}
```

## Files

- `files.includes`: file include and exclude patterns used before formatting,
  checking, or linting CLI targets. Patterns are evaluated relative to the
  config file directory when possible.
- Use `!` to exclude a pattern after including broader targets, for example
  `["**/*.html.erb", "!vendor/**"]`.
- Supported wildcards are `*`, `?`, and `**`.
- When `files.includes` is omitted, erbfmt processes every file explicitly
  passed on the command line.

## Parser

- `parser.allowHtmlOptionalClosingTags`: allow common HTML optional closing
  tags such as omitted `</li>`, `</p>`, `</td>`, and `</tr>`. The default is
  `false`; erbfmt normally treats omitted HTML close tags as parse errors so
  ERB-heavy templates do not get reformatted under an inferred tree. When set
  to `true`, erbfmt accepts these forms but preserves the omitted closing tags
  instead of inserting them.

## Formatter

- `formatter.enabled`: enable or disable formatting.
- `formatter.indentStyle`: `space` or `tab`.
- `formatter.indentWidth`: number of spaces per indent level when using
  `space`.
- `formatter.indentHtml`: indent nested HTML tags.
- `formatter.lineEnding`: `lf` or `crlf`.
- `formatter.lineWidth`: target line width. Opening, void, and self-closing
  HTML tags that exceed this width are expanded one attribute per line, with
  the closing marker on its own line. For format-sensitive subtrees such as
  `pre`, `svg`, `math`, `template`, `noscript`, `contenteditable`, and inline
  `white-space` styles, erbfmt preserves the subtree content but may still wrap
  the opening tag attributes. Standalone ERB tags that exceed this width are
  expanded by moving `<%` / `<%=` and `%>` onto their own lines. When the Ruby
  code is a simple method call with top-level comma-separated arguments, erbfmt
  may place one argument per line. Command-style calls gain explicit
  parentheses; calls that already have parentheses retain them. Ruby
  expressions that cannot be recognized safely are left intact.
- `formatter.trailingNewline`: keep or remove the final newline. The default
  `true` follows normal source-file conventions. Use `false` for ERB files that
  are intentionally rendered as inline partial fragments where a final newline
  would become visible whitespace in the surrounding output.

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
- `linter.rules.noNonDoubleQuotedHtmlAttributeValue`: `error`, `warn`, or
  `off`.
- `linter.rules.noSelfClosingHtmlTag`: `error`, `warn`, or `off`.
- `linter.rules.unsupportedErbBlockStarter`: `error`, `warn`, or `off`.

`error` diagnostics make `erbfmt --lint` exit with a failure status. `warn`
diagnostics are reported, but warning-only lint results exit successfully.
VSCode diagnostics use the matching warning or error severity.

See [LintRules.md](LintRules.md) for the current lint rule design and the next
planned rules.

See [Ignore.md](Ignore.md) for `erbfmt-ignore` lint and formatter directives.
