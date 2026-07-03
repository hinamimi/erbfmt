# Forem Audit

This document records the formatter audit against the Forem Rails views.

The local clone is intentionally ignored by git:

```text
external/rails-repos/forem
```

## Baseline

```bash
find external/rails-repos/forem/app/views -name '*.erb' | wc -l
```

Result:

```text
782 ERB files
```

## Audit Commands

```bash
rm -rf /tmp/erbfmt-forem-audit
mkdir -p /tmp/erbfmt-forem-audit/original /tmp/erbfmt-forem-audit/formatted
cp -a external/rails-repos/forem/app/views /tmp/erbfmt-forem-audit/original/views
cp -a external/rails-repos/forem/app/views /tmp/erbfmt-forem-audit/formatted/views
cargo run --quiet -- --write /tmp/erbfmt-forem-audit/formatted/views > /tmp/erbfmt-forem-audit/write.log 2>&1
diff -ru /tmp/erbfmt-forem-audit/original/views /tmp/erbfmt-forem-audit/formatted/views > /tmp/erbfmt-forem-audit/views.diff
diff -qr /tmp/erbfmt-forem-audit/original/views /tmp/erbfmt-forem-audit/formatted/views
```

## Initial Result

Before the raw text lexer fix, Forem exposed a parser gap where ERB tags inside
`script` or `style` raw text split the surrounding HTML fragment. That made the
HTML tokenizer inspect JavaScript and CSS as HTML.

```text
54 files failed to parse.
```

Representative examples:

```erb
<script>
  if (numTags < 6) {
    document.body.classList.add("user-tags-followed-" + numTags);
  }
  link.href = '<%= asset_path "OpenDyslexic-Regular.otf" %>';
</script>
```

```erb
<style>
  :root {
    --accent-brand-rgb: <%= brand_color %>;
  }
</style>
```

## Raw Text Fix

Status: fixed.

The lexer now treats ERB tags inside `script` and `style` as part of the HTML
fragment instead of tokenizing them as standalone ERB tags. This is intentionally
conservative: raw text content is format-sensitive and frequently contains
JavaScript or CSS syntax that resembles HTML.

After the fix:

```text
50 files failed to parse.
699 files were written by formatting.
571 files changed by formatting.
40280 unified diff lines.
```

The raw text failures in files such as these no longer occur:

- `app/views/agent_sessions/new.html.erb`
- `app/views/credits/new.html.erb`
- `app/views/layouts/_user_config.html.erb`
- `app/views/articles/show.html.erb`

## Remaining Parse Failures

The remaining failures appear to be different from the raw text gap. They fall
mostly into these categories.

### Partial HTML Fragments

Some partials intentionally open or close HTML elements that are balanced by the
caller or by another partial. erbfmt currently parses each file as a standalone
template, so these fragments fail with unexpected close tags or unclosed open
tags.

Examples:

- `app/views/credits/_pricing.en.html.erb`
- `app/views/credits/_pricing.fr.html.erb`
- `app/views/credits/_pricing.pt.html.erb`
- `app/views/admin/articles/_article_item.html.erb`

### Cross-ERB-Block HTML Structure

Some templates open an HTML element inside one ERB control-flow block and close
it inside a separate ERB block. This is valid as rendered Rails output, but it is
hard for a structural formatter to prove safe at file scope.

Example:

```erb
<% unless internal_navigation? %>
  <body ...>
<% end %>

...

<% unless internal_navigation? %>
  </body>
  </html>
<% end %>
```

Observed in:

- `app/views/layouts/application.html.erb`

### Invalid or Non-Canonical HTML

Some failures appear to be real HTML structure issues or intentional legacy
markup, such as mismatched heading tags, paragraph/block nesting, or `</br>`
inside `pre` content.

Examples:

- `app/views/admin/organizations/_activity.html.erb`
- `app/views/admin/tags/edit.html.erb`
- `app/views/pages/_editor_guide_text.en.html.erb`
- `app/views/pages/_editor_guide_text.fr.html.erb`
- `app/views/pages/_editor_guide_text.pt.html.erb`

### ERB Control Flow Outside a Single Parseable Structure

At least one file contains an ERB `end` that does not have a matching block
starter from the parser's point of view.

Example:

- `app/views/liquid_embeds/show.html.erb`

## Optional Closing Tags

Testing with `parser.allowHtmlOptionalClosingTags: true` did not improve the
Forem audit. It increased the remaining parse failures in this snapshot because
many failures are not plain HTML optional close cases; they are partial
fragments, invalid HTML, or cross-block structures.

For this corpus, optional close support should not be treated as the main path to
full parse coverage.

## Safety Notes

The changed files should still be reviewed with the same safety lens used for
the OpenStreetMap audit:

- inline text/helper adjacency
- inline element adjacency
- punctuation next to ERB output
- format-sensitive raw text
- multiline ERB output next to visible text

The raw text fix specifically reduces risk by keeping `script` and `style`
contents intact when they contain ERB.

## Next Candidates

The remaining Forem failures are not all good formatter targets. The most useful
future work is likely:

1. Add a documented "fragment mode" design for partials that intentionally start
   or end with unmatched HTML.
2. Decide whether cross-ERB-block HTML open/close pairs should be supported,
   ignored, or linted as unsafe for formatting.
3. Review the large diff for visual-risk patterns and promote any concrete issue
   into formatter fixtures.
