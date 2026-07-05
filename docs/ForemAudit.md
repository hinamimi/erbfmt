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

## `allowUnbalancedHtmlFragments` Result

Testing with `parser.allowUnbalancedHtmlFragments: true` reduced the remaining
Forem parse failures, but it did not make the whole corpus parseable:

```text
44 files failed to parse.
705 files were written by formatting.
577 files changed by formatting.
40587 unified diff lines.
```

This is expected. The option only allows close tags that belong to an outer
template and open tags that are closed by a caller. It does not hide internal
mismatched HTML tags or ERB block errors.

### Partial HTML Fragments

Some partials intentionally open or close HTML elements that are balanced by the
caller or by another partial. erbfmt currently parses each file as a standalone
template by default, so these fragments fail with unexpected close tags or
unclosed open tags unless `parser.allowUnbalancedHtmlFragments` is enabled.

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

Formatter policy: do not format these structures by inferring rendered control
flow. They should remain parse failures, or users should opt out with
`erbfmt-ignore` when the pattern is intentional. `parser.allowUnbalancedHtmlFragments`
is for partial fragments at file boundaries and should not be used to make
cross-ERB-block HTML structures parseable.

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

## Formatter Coverage Improvements

After reviewing long lines that remained in the formatted Forem copy, one safe
formatting gap stood out: Rails helper arguments that contain Active Record style
method chains.

Example:

```erb
<%= render partial: "layouts/sidebar_nav_link", collection: NavigationLink.where(id: @navigation_links[:default_nav_ids]).where.not(name: ["Home", "Reading List"]).ordered, as: :link %>
```

The Ruby AST now accepts receiver method calls such as `.where.not(...)`, and the
formatter can wrap long method-chain arguments while keeping the helper call
safe and readable:

```erb
<%= render(
  partial: "layouts/sidebar_nav_link",
  collection: NavigationLink
    .where(id: @navigation_links[:default_nav_ids])
    .where.not(name: ["Home", "Reading List"])
    .ordered,
  as: :link
) %>
```

In the local Forem audit copy, this reduced long Rails-helper lines from 133 to
124 with the same filter used during the review.

Two additional safe formatting gaps were then addressed:

- existing multiline command-style helpers, when every continuation line is a
  comma-separated argument and the expression does not use legacy hashrocket
  options
- ternary expressions inside helper arguments, while keeping top-level ternary
  expressions out of formatter rewriting

Examples:

```erb
<%= link_to "Delete", admin_badge_badge_automation_path(@badge, automation), 
    method: :delete, 
    data: { confirm: "Are you sure?" },
    class: "crayons-btn crayons-btn--danger crayons-btn--s" %>
```

```erb
<%= link_to(
  "Delete",
  admin_badge_badge_automation_path(@badge, automation),
  method: :delete,
  data: { confirm: "Are you sure?" },
  class: "crayons-btn crayons-btn--danger crayons-btn--s"
) %>
```

```erb
<li><%= link_to(
  t("views.admin.users.tabs.overview"),
  admin_user_path(@user.id),
  class: "crayons-navigation__item #{'crayons-navigation__item crayons-navigation__item--current' if @current_tab == 'overview'}",
  aria: @current_tab == "overview" ? { current: "page" } : {}
) %></li>
```

After these changes, the same local Forem review filter reduced long
Rails-helper lines from 119 to 111. Applying the formatter a second time to that
copy produced no further diff.

Another Milestone C gap was receiver method calls with brace blocks inside
helper arguments. Forem has several select helpers whose option collections are
built with `keys.map { |value| ... }`. These are safe to keep as a single Ruby
argument while still wrapping the surrounding helper call.

Example:

```erb
<%= f.select :type_of, Survey.type_ofs.keys.map { |type| [type.titleize, type] }, {}, class: "crayons-select__native" %>
```

```erb
<%= f.select(
  :type_of,
  Survey.type_ofs.keys.map { |type| [type.titleize, type] },
  {},
  class: "crayons-select__native"
) %>
```

The Ruby AST now parses receiver method calls with brace block suffixes, such as
`Survey.type_ofs.keys.map { |type| ... }`, and the formatter can use that parse
as part of the helper-call safety check. In the local Forem audit copy, this
reduced the same long Rails-helper line filter from 111 to 107. Applying the
formatter a second time to that copy produced no further diff.

Two more Ruby wrapping gaps were then addressed:

- helper calls with top-level `if` / `unless` modifier suffixes, such as
  `render ... unless user_signed_in?`
- quoted label keyword arguments, such as `"aria-label": ...` and
  `"data-action": ...`

Examples:

```erb
<%= render partial: "shared/auth_widget", locals: { tracking_id: "ca_left_sidebar_home_page", source: "left_sidebar" } unless user_signed_in? %>
```

```erb
<%= render(
  partial: "shared/auth_widget",
  locals: {
    tracking_id: "ca_left_sidebar_home_page",
    source: "left_sidebar"
  }
) unless user_signed_in? %>
```

```erb
<%= link_to "‹", calendar_path(month: prev_month.strftime("%Y-%m"), start: params[:start].presence, end: params[:end].presence, type_of: params[:type_of].presence), class: "calendar__nav-btn", "aria-label": t("views.calendar.nav.previous_month") %>
```

```erb
<%= link_to(
  "‹",
  calendar_path(month: prev_month.strftime("%Y-%m"), start: params[:start].presence, end: params[:end].presence, type_of: params[:type_of].presence),
  class: "calendar__nav-btn",
  "aria-label": t("views.calendar.nav.previous_month")
) %>
```

The modifier suffix support reduced the local Forem long Rails-helper filter
from 107 to 105. Quoted label keyword support reduced it further from 105 to 96.
Both passes were idempotent when the formatter was applied a second time.

Nested helper calls inside keyword arguments were then improved. This does not
always reduce the simple long-line filter, because the outer call was already
split, but it makes nested Rails helpers much easier to read and gives them
consistent indentation.

Example:

```erb
<%= t("views.admin.users.actions.adjust_credit.selector_html",
      action: f.select(:credit_action, options_for_select({ t("views.admin.users.actions.adjust_credit.add") => "Add" }), {}, class: "js-credit-action crayons-select w-auto"),
      amount: f.text_field(:credit_amount,
                           type: "number",
                           required: true,
                           class: "js-credit-amount crayons-textfield w-auto",
                           value: 1,
                           size: 5,
                           min: 1,
                           max: 9999),
      credits: f.label(:credit_amount, t("views.admin.users.actions.adjust_credit.unit"), class: "color-secondary flex-1", "aria-label": t("views.admin.users.actions.adjust_credit.aria_credits"))) %>
```

```erb
<%= t(
  "views.admin.users.actions.adjust_credit.selector_html",
  action: f.select(
    :credit_action,
    options_for_select({ t("views.admin.users.actions.adjust_credit.add") => "Add" }),
    {},
    class: "js-credit-action crayons-select w-auto"
  ),
  amount: f.text_field(
    :credit_amount,
    type: "number",
    required: true,
    class: "js-credit-amount crayons-textfield w-auto",
    value: 1,
    size: 5,
    min: 1,
    max: 9999
  ),
  credits: f.label(
    :credit_amount,
    t("views.admin.users.actions.adjust_credit.unit"),
    class: "color-secondary flex-1",
    "aria-label": t("views.admin.users.actions.adjust_credit.aria_credits")
  )
) %>
```

In the local Forem audit copy, this changed 37 files and remained idempotent on
the second formatter pass. The same long Rails-helper filter stayed at 96,
because this pass primarily improved nested indentation rather than top-level
line count.

## Next Candidates

The remaining Forem failures are not all good formatter targets. The most useful
future work is likely:

1. Review the `parser.allowUnbalancedHtmlFragments` diff for visual-risk
   patterns before recommending it broadly. This option is intentionally
   disabled by default.
2. Decide whether cross-ERB-block HTML open/close pairs should be supported,
   ignored, or linted as unsafe for formatting.
3. Review the large diff for visual-risk patterns and promote any concrete issue
   into formatter fixtures.
