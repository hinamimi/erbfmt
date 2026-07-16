# erbfmt v0.3.1

erbfmt v0.3.1 improves HTML attribute linting for static HTML tags.

## Highlights

- Added `noUnknownHtmlAttribute`, which reports unknown HTML attributes and
  suggests the canonical HTML spelling for common typos such as `tabIndex`,
  `tab-index`, and `auto-complete`.
- Added `noDeprecatedHtmlAttribute`, which reports obsolete presentation and
  legacy attributes such as `align`, `summary`, `bgcolor`, and `frameborder`.
  This rule is `info` by default so existing email and embedded HTML can adopt
  it gradually.
- Expanded known attribute support for frequent form, link, media, embed,
  list, and table elements.
- Avoided false positives for global attributes, `data-*`, `aria-*`, common
  framework attributes, custom elements, SVG/MathML, and tags with ERB
  attribute fragments.

## Configuration

Both rules can be configured independently in `erbfmt.json`:

```json
{
  "linter": {
    "rules": {
      "noUnknownHtmlAttribute": "warn",
      "noDeprecatedHtmlAttribute": "info"
    }
  }
}
```

Use `"off"`, `"info"`, `"warn"`, or `"error"` to match the project's
adoption policy. Only `error` diagnostics make `erbfmt --lint` exit with a
failure status.

## Upgrade Notes

`noUnknownHtmlAttribute` is enabled as a warning by default. It can reveal
previously unnoticed spelling mistakes in static HTML, while warning-only lint
results continue to exit successfully. Disable the rule or adjust its severity
if a project relies on non-standard attributes that are not covered by the
framework attribute allowances.
