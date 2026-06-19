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

## Formatter Ignore Contract

Formatter ignore is not implemented yet. The first implementation will use the
following syntax:

```erb
<!-- erbfmt-ignore format: third-party markup -->
<div   class="legacy"><span>Keep   this spacing</span></div>
```

```erb
<%# erbfmt-ignore format: generated helper call %>
<%= render "cards/card",   card: card %>
```

`format` and `lint` are separate selectors. To ignore both formatting and lint
diagnostics, use two directives instead of combining selectors in one comment.

The initial formatter scope is deliberately narrow:

- The directive must be the only content on its physical line.
- The target must begin on the immediately following physical line.
- The target is the next non-whitespace AST node in the same parent, including
  its complete subtree. An HTML element therefore includes its closing tag and
  descendants; an ERB control-flow block includes its branches and `<% end %>`.
- The target must begin after indentation only. Inline fragments and directives
  inside HTML opening tags are not supported.
- The target source lines are preserved byte-for-byte, including indentation,
  internal whitespace, and line endings.
- An invalid or ambiguous directive is ignored, and normal formatting applies.

Both `erbfmt-ignore format` and `erbfmt-ignore-next-line format` will use this
same immediately-following-line behavior.

See [FormatterIgnoreDesign.md](FormatterIgnoreDesign.md) for the source-range
and mixed AST changes required before implementation.

## Current Scope

Ignore directives currently affect lint diagnostics only.

Lexer and parser diagnostics, such as unterminated ERB tags or mismatched HTML
close tags, are not suppressible because erbfmt cannot safely continue with an
invalid document structure.

Formatter ignore remains inactive until the source-range work described above
is implemented. A directive using the `format` selector currently has no effect.
