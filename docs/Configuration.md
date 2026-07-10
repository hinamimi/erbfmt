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

The shape is intentionally close to Biome's `formatter`, `linter`, and
`assist` configuration sections, with a small erbfmt-specific `parser` section
and `formatter.indentHtml` option for ERB and HTML-specific behavior.

## Example

```json
{
  "files": {
    "includes": ["**/*.html.erb", "!vendor/**"]
  },
  "parser": {
    "allowHtmlOptionalClosingTags": false,
    "allowUnbalancedHtmlFragments": false
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
      "noDeprecatedHtmlAttribute": "info",
      "noDeprecatedHtmlTag": "error",
      "noDuplicateHtmlAttribute": "error",
      "noInvalidHtmlBooleanAttribute": "error",
      "noInvalidHtmlNesting": "error",
      "noNonDoubleQuotedHtmlAttributeValue": "error",
      "noSelfClosingHtmlTag": "error",
      "noUnknownHtmlAttribute": "warn",
      "unsupportedErbBlockStarter": "error"
    }
  },
  "assist": {
    "enabled": false,
    "includes": ["**/*.html.erb"],
    "actions": {
      "html.useDoubleQuotedAttributes": true,
      "html.fixSelfClosingTags": true,
      "html.simplifyBooleanAttributes": true,
      "erb.removeEmptyTags": true
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
  passed on the command line. For recursive directory discovery, erbfmt uses
  `*.html.erb` files by default.

## Parser

- `parser.allowHtmlOptionalClosingTags`: allow common HTML optional closing
  tags such as omitted `</li>`, `</p>`, `</td>`, and `</tr>`. The default is
  `false`; erbfmt normally treats omitted HTML close tags as parse errors so
  ERB-heavy templates do not get reformatted under an inferred tree. When set
  to `true`, erbfmt accepts these forms but preserves the omitted closing tags
  instead of inserting them.
- `parser.allowUnbalancedHtmlFragments`: allow partial HTML fragments that have
  close tags from an outer template or open tags that are closed by a caller.
  The default is `false`. When set to `true`, erbfmt preserves unmatched close
  tags as text and allows unclosed HTML tags at the end of the file. Internal
  mismatched tags still fail because they are unsafe to reformat structurally.
  This option is not intended for HTML open/close pairs that cross separate ERB
  blocks; those templates should remain parse failures or be handled with
  `erbfmt-ignore`.

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
- `linter.rules.emptyErbBranch`: `error`, `warn`, `info`, or `off`.
- `linter.rules.emptyErbCodeTag`: `error`, `warn`, `info`, or `off`.
- `linter.rules.emptyErbControlBlock`: `error`, `warn`, `info`, or `off`.
- `linter.rules.noDeprecatedHtmlAttribute`: `error`, `warn`, `info`, or
  `off`. The default severity is `info`.
- `linter.rules.noDeprecatedHtmlTag`: `error`, `warn`, `info`, or `off`.
- `linter.rules.noDuplicateHtmlAttribute`: `error`, `warn`, `info`, or `off`.
- `linter.rules.noInvalidHtmlBooleanAttribute`: `error`, `warn`, `info`, or
  `off`.
- `linter.rules.noInvalidHtmlNesting`: `error`, `warn`, `info`, or `off`.
- `linter.rules.noNonDoubleQuotedHtmlAttributeValue`: `error`, `warn`, `info`,
  or `off`.
- `linter.rules.noSelfClosingHtmlTag`: `error`, `warn`, `info`, or `off`.
- `linter.rules.noUnknownHtmlAttribute`: `error`, `warn`, `info`, or `off`.
  The default severity is `warn`. The initial implementation covers frequent
  HTML tags such as `input`, `a`, `button`, `form`, `label`, `img`, `script`,
  `link`, `meta`, `textarea`, `select`, `option`, media/embed tags, and common
  list/table tags, skips tags with ERB attribute fragments, and allows
  `data-*`, `aria-*`, and common framework attributes such as `v-*`, `x-*`,
  and `hx-*`.
- `linter.rules.unsupportedErbBlockStarter`: `error`, `warn`, `info`, or
  `off`.

`error` diagnostics make `erbfmt --lint` exit with a failure status. `warn` and
`info` diagnostics are reported, but warning/info-only lint results exit
successfully. VSCode diagnostics use the matching info, warning, or error
severity.

See [LintRules.md](LintRules.md) for the current lint rule design and the next
planned rules.

## Assist

The `assist` section configures safe code actions. Assist is separate from
formatting and linting: linter rules report problems, while assist actions
offer edits that erbfmt can apply without needing Ruby or Rails semantic
analysis.

- `assist.enabled`: enable or disable assist actions. The default is `false`.
  `assist.enable` is also accepted as an alias. The CLI `--assist` mode runs
  assist explicitly even when the default config keeps this value `false`.
- `assist.includes`: file include and exclude patterns used by assist actions.
  The pattern syntax matches `files.includes`.
- `assist.actions`: action-name to boolean map. Known actions default to enabled
  when assist runs, unless explicitly set to `false`.

Current actions:

- `html.useDoubleQuotedAttributes`: replace single-quoted or unquoted HTML
  attribute values with double-quoted values when the value itself does not
  contain a double quote.
- `html.fixSelfClosingTags`: replace `<br />` with `<br>` and `<div />` with
  `<div></div>`.
- `html.simplifyBooleanAttributes`: replace redundant boolean attributes such
  as `checked="checked"` with `checked`. Values such as `disabled="false"` are
  not changed because they may indicate a semantic bug.
- `erb.removeEmptyTags`: remove empty `<% %>` and `<%= %>` tags.

CLI examples:

```bash
erbfmt --assist app/views
erbfmt --assist --assist-format json app/views
erbfmt --assist --write app/views
```

See [Ignore.md](Ignore.md) for `erbfmt-ignore` lint and formatter directives.
