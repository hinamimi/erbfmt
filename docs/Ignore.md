# Ignore Directives

erbfmt supports line-based lint ignore directives in HTML comments and ERB
comments.

The directive applies to the next physical line.

## Ignore All Lint Rules On The Next Line

```erb
<!-- erbfmt-ignore lint: legacy markup from upstream -->
<center>Legacy</center>
```

```erb
<%# erbfmt-ignore lint: generated placeholder %>
<% %>
```

`erbfmt-ignore-next-line` is also accepted:

```erb
<!-- erbfmt-ignore-next-line lint: legacy markup -->
<center>Legacy</center>
```

## Ignore One Rule On The Next Line

Use `lint/<ruleName>` to suppress only one rule.

```erb
<!-- erbfmt-ignore lint/noDeprecatedHtmlTag: legacy markup -->
<center>Legacy</center>
```

The rule name is the same camelCase name used in `erbfmt.json`, such as:

- `emptyErbBranch`
- `emptyErbCodeTag`
- `emptyErbControlBlock`
- `noDeprecatedHtmlTag`
- `noDuplicateHtmlAttribute`
- `noInvalidHtmlBooleanAttribute`
- `noInvalidHtmlNesting`
- `noSelfClosingHtmlTag`
- `unsupportedErbBlockStarter`

## Scope

Ignore directives currently affect lint diagnostics only.

Lexer and parser diagnostics, such as unterminated ERB tags or mismatched HTML
close tags, are not suppressible because erbfmt cannot safely continue with an
invalid document structure.

Formatter ignore is intentionally not implemented yet. Formatting ignore needs
source ranges in the mixed AST so erbfmt can preserve the original source for a
subtree without guessing.
