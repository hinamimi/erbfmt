# Formatter Ignore Design

This document records the implementation boundary for formatter ignore. The
user-facing syntax and scope are defined in [Ignore.md](Ignore.md).

## Decision

Formatter ignore targets one complete non-whitespace mixed AST node/subtree,
starting on the physical line immediately after the directive. Whitespace-only
HTML text between the directive and target is trivia, not the target. Formatter
ignore does not target an arbitrary line range.

This keeps structural formatting predictable:

- An HTML element is preserved from its opening tag through its closing tag.
- A standalone ERB tag is preserved as one node.
- An ERB control-flow block is preserved through all branches and `<% end %>`.
- Whitespace-sensitive inline fragments and individual HTML attributes remain
  outside the initial scope.

## Directive Parsing

The formatter accepts these selectors in standalone HTML or ERB comments:

```erb
<!-- erbfmt-ignore format: reason -->
<%# erbfmt-ignore format: reason %>
```

`erbfmt-ignore-next-line format` is an alias with the same behavior. Existing
`lint` and `lint/<ruleName>` selectors keep their current semantics. One comment
contains one selector; users place two directives when both formatter and lint
suppression are needed.

The directive parser should be shared by formatter and linter so selector and
reason parsing do not drift.

## Source Ranges

Lexer tokens already contain absolute byte spans, while HTML tokens contain
spans relative to their HTML fragment. The mixed AST currently discards those
spans and therefore cannot reproduce a subtree from the original source.

Implementation requires:

1. Add an optional absolute byte range to mixed AST nodes.
2. Convert relative HTML token ranges to absolute ranges using the containing
   lexer token's `span.start`.
3. Store opening ranges in HTML and ERB parser frames, then close each range at
   the matching HTML close tag or ERB block end token.
4. Represent ERB comments explicitly. They currently pass through the lexer as
   ordinary ERB code, which cannot preserve the `<%#` marker safely.
5. Pass the original source to the formatter alongside the parsed document.

Unspanned parser entry points used by focused unit tests may keep `None` ranges.
Formatter ignore is applied only when a complete valid range is available.

## Preservation

For a valid directive, the formatter copies the complete physical lines that
cover the target range from the original source. This preserves leading
indentation, internal whitespace, line endings, and the target subtree exactly.

Ignored source takes precedence over `formatter.lineEnding`. The current final
whole-output line-ending replacement must be replaced by generated-line output
that uses the configured ending without rewriting copied raw source.

The target is eligible only when:

- the directive occupies a standalone physical line;
- the target starts on the immediately following line after indentation only;
- the target range ends before any non-whitespace sibling content on its final
  physical line; and
- the directive and target belong to the same AST parent.

If any condition cannot be proven from source ranges, normal formatting applies.

## Initial Tests

The implementation milestone should cover:

- HTML comment followed by a complete HTML element subtree;
- ERB comment followed by a long standalone ERB output tag;
- ERB comment followed by an ERB control-flow block;
- surrounding nodes still being formatted normally;
- a blank line, inline target, or malformed selector not suppressing formatting;
- idempotency after an ignored subtree is preserved; and
- lint directives retaining their existing behavior.

The proposed fixture is `samples/formatter-ignore-next.html.erb`.
