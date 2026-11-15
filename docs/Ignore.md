# Ignore Directives

erbfmt supports lint and formatter ignore directives in HTML comments and ERB
comments.

Place the directive on its own line immediately before the target, without a
blank line between them. HTML comments (`<!-- ... -->`) and ERB comments
(`<%# ... %>`) accept the same selectors. ERB comments do not appear in the
rendered HTML.

| Selector | Effect |
| --- | --- |
| `format` | Preserves the complete element or ERB block starting on the next physical line. |
| `lint` | Suppresses all lint diagnostics located on the next physical line. |
| `lint/<ruleName>` | Suppresses only the named lint rule on the next physical line. |
| `all` | Combines `format` preservation with `lint` suppression on the next physical line. |

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
- `noDeprecatedHtmlAttribute`
- `noDeprecatedHtmlTag`
- `noDuplicateHtmlAttribute`
- `noInvalidHtmlBooleanAttribute`
- `noInvalidHtmlNesting`
- `noNonDoubleQuotedHtmlAttributeValue`
- `noSelfClosingHtmlTag`
- `noUnknownHtmlAttribute`
- `unsupportedErbBlockStarter`

## Formatter Ignore Contract

Formatter ignore uses the following syntax:

```erb
<!-- erbfmt-ignore format: third-party markup -->
<div   class="legacy"><span>Keep   this spacing</span></div>
```

```erb
<%# erbfmt-ignore format: generated helper call %>
<%= render "cards/card",   card: card %>
```

Use the `all` selector to preserve the next complete node and suppress all lint
diagnostics located on the immediately following physical line:

```erb
<!-- erbfmt-ignore all: generated markup -->
<div   class="generated"><center>Keep this</center></div>
```

For a multiline target, `all` preserves the whole subtree, but lint diagnostics
on later lines are still reported.

### Preserve A Multiline HTML Element

For example, keep a third-party widget's spacing while formatting the surrounding
template. Running `erbfmt app/views/widgets/show.html.erb` on this input:

```erb
<section>
  <!-- erbfmt-ignore format: preserve third-party widget -->
    <div   class="widget">
 <span>Keep this indentation</span>
    </div>
<p>Format this paragraph</p>
</section>
```

produces:

```erb
<section>
  <!-- erbfmt-ignore format: preserve third-party widget -->
    <div   class="widget">
 <span>Keep this indentation</span>
    </div>
  <p>Format this paragraph</p>
</section>
```

The entire `div`, including its closing tag and original indentation, is
preserved. The following `p` is formatted normally.

### Preserve An ERB Control-Flow Block

Place an ERB comment immediately before the opening tag to preserve every branch
through the matching `<% end %>`:

```erb
<%# erbfmt-ignore format: preserve generated conditional %>
<% if show_widget? %>
    <%= render "widgets/widget",   widget: widget %>
<% else %>
 <p>Widget unavailable</p>
<% end %>
```

This keeps the entire block as written. A `format` directive does not suppress
lint diagnostics inside the block.

### A Blank Line Prevents Formatter Ignore

The following directive has no effect because the target does not start on the
immediately following physical line:

```erb
<!-- erbfmt-ignore format: preserve third-party widget -->

<div   class="widget"><span>Formatted normally</span></div>
```

### Scope Restrictions

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

Both `erbfmt-ignore format` and `erbfmt-ignore-next-line format` use this
same immediately-following-line behavior.

See [FormatterIgnoreDesign.md](FormatterIgnoreDesign.md) for the source-range
and mixed AST implementation details.

## Diagnostics Scope

Lexer and parser diagnostics, such as unterminated ERB tags or mismatched HTML
close tags, are not suppressible because erbfmt cannot safely continue with an
invalid document structure.
