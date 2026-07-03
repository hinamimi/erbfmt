# openstreetmap-website formatter audit

Date: 2026-07-03

This note records an `erbfmt 0.3.0-dev` audit against
`external/rails-repos/openstreetmap-website`.

The target repository is kept outside the committed source tree. It is used as
a real Rails view corpus for formatter safety checks.

## Commands

```bash
cd external/rails-repos/openstreetmap-website
erbfmt --version
erbfmt --check app/views
```

Version:

```text
erbfmt 0.3.0-dev
```

To inspect formatting diffs without touching the external repository:

```bash
rm -rf /tmp/erbfmt-osm-audit
mkdir -p /tmp/erbfmt-osm-audit/original /tmp/erbfmt-osm-audit/formatted
cp -a app/views /tmp/erbfmt-osm-audit/original/views
cp -a app/views /tmp/erbfmt-osm-audit/formatted/views
erbfmt --write /tmp/erbfmt-osm-audit/formatted/views
diff -ru /tmp/erbfmt-osm-audit/original/views /tmp/erbfmt-osm-audit/formatted/views
```

## `--check` result

`erbfmt --check app/views` produced 241 result lines.

```text
73 files are formatted.
166 files are not formatted.
2 files failed to parse.
```

The initial parse errors were:

```text
Error: failed to parse `app/views/layers_panes/show.html.erb`: unclosed HTML tag `<div class="overlay-layers p-3">` at line 20, column 3
Error: failed to parse `app/views/notes/index.html.erb`: mismatched HTML close tag `</tr>`, expected `</tr<%>` at line 61, column 5
```

Running `--write` on a temporary copy wrote 239 files, failed on the same 2
files, and changed 166 files.

Note: the counts above are from the initial `erbfmt 0.3.0-dev` audit run. Later
parser and formatter safety work has mitigated several issues recorded below;
this document keeps both the original finding and the current status.

Current verification:

```bash
cargo run --quiet -- --check external/rails-repos/openstreetmap-website/app/views
```

now reports only normal formatted / not formatted results for the view corpus.
The two parser failures from the initial audit no longer reproduce.

Second diff audit:

```bash
rm -rf /tmp/erbfmt-osm-audit
mkdir -p /tmp/erbfmt-osm-audit/original /tmp/erbfmt-osm-audit/formatted
cp -a external/rails-repos/openstreetmap-website/app/views /tmp/erbfmt-osm-audit/original/views
cp -a external/rails-repos/openstreetmap-website/app/views /tmp/erbfmt-osm-audit/formatted/views
cargo run --quiet -- --write /tmp/erbfmt-osm-audit/formatted/views
diff -ru /tmp/erbfmt-osm-audit/original/views /tmp/erbfmt-osm-audit/formatted/views > /tmp/erbfmt-osm-audit/views.diff
diff -qr /tmp/erbfmt-osm-audit/original/views /tmp/erbfmt-osm-audit/formatted/views > /tmp/erbfmt-osm-audit/changed-files.txt
```

Result after parser fixes and multiline setup-opener preservation:

```text
0 files failed to parse.
130 files changed by formatting.
4693 unified diff lines.
```

Third diff audit after inline-safety and hashrocket preservation fixes:

```bash
rm -rf /tmp/erbfmt-osm-audit
mkdir -p /tmp/erbfmt-osm-audit/original /tmp/erbfmt-osm-audit/formatted
cp -a external/rails-repos/openstreetmap-website/app/views /tmp/erbfmt-osm-audit/original/views
cp -a external/rails-repos/openstreetmap-website/app/views /tmp/erbfmt-osm-audit/formatted/views
cargo run --quiet -- --write /tmp/erbfmt-osm-audit/formatted/views
diff -ru /tmp/erbfmt-osm-audit/original/views /tmp/erbfmt-osm-audit/formatted/views > /tmp/erbfmt-osm-audit/views.diff
diff -qr /tmp/erbfmt-osm-audit/original/views /tmp/erbfmt-osm-audit/formatted/views > /tmp/erbfmt-osm-audit/changed-files.txt
```

Result after the latest safety fixes:

```text
0 files failed to parse.
88 files changed by formatting.
2689 unified diff lines.
```

Fourth diff audit after legacy hash option folding:

```text
0 files failed to parse.
160 files changed by formatting.
6839 unified diff lines.
```

This increased the diff size because single-line old-style hash option helpers
are now folded when the Ruby AST can classify them as safe call arguments.

## Parser issues found

### Multiline ERB block opener with Ruby setup

Status: fixed.

`app/views/layers_panes/show.html.erb` contains an ERB block opener where the
Ruby code spans multiple lines and performs setup before the ERB tag closes.

```erb
<div class="overlay-layers p-3">
  <p class="text-body-secondary small mb-2"><%= t "javascripts.map.layers.overlays" %></p>
  <% @overlay_layers.each do |layer|
       options = { :data => layer, :class => "form-check" }
       options[:title] = t(".map_#{layer[:name]}_zoom_in_tooltip") if layer[:max_area] %>
    <%= tag.div(**options) do %>
      <%= tag.label :id => "label-layers-#{layer[:name]}", :class => "form-check-label" do %>
        <input type="checkbox" class="form-check-input">
        <%= t "javascripts.map.layers.#{layer[:name]}" %>
      <% end %>
    <% end %>
  <% end %>
</div>
```

The current parser treats the outer `<div>` as unclosed. This suggests that the
mixed parser is not recognizing this multiline `<% ... do |layer| ... %>` as an
ERB block opener.

Original safety impact:

- Formatter does not run for the file.
- This is safer than rewriting incorrectly, but the diagnostic points at the
  outer HTML tag instead of the unsupported ERB block shape.

Fix:

- Recognize an ERB code tag as a `Do` block opener when its first non-empty
  line ends with a `do` block opener, even if later setup statements remain in
  the same ERB tag.
- Keep self-contained Ruby control flow with an internal `end` as plain ERB
  code.
- Preserve the full ERB tag body, so the formatter can remain conservative
  about the setup statements.
- When source ranges are available, preserve the original multiline opener
  shape instead of normalizing the setup statements.

Verification:

```bash
cargo run --quiet -- --check external/rails-repos/openstreetmap-website/app/views/layers_panes/show.html.erb
```

now reaches the normal format-check result:

```text
external/rails-repos/openstreetmap-website/app/views/layers_panes/show.html.erb: file is not formatted.
```

The second diff audit confirms that the setup opener is no longer rewritten.
The surrounding HTML may still be formatted, but this opener remains source-like:

```erb
<% @overlay_layers.each do |layer|
     options = { :data => layer, :class => "form-check" }
     options[:title] = t(".map_#{layer[:name]}_zoom_in_tooltip") if layer[:max_area] %>
```

### Trim marker confused with an HTML close tag

Status: fixed. The original diagnosis blamed a table row loop ending with
`<% end -%>`, but the actual trigger was a conditional attribute immediately
after the tag name:

```erb
<tr<% if note.author == @user %> class="table-primary"<% end %>>
```

Before the fix, the HTML tokenizer read the opening tag name as `tr<%`. The
later `</tr>` close tag then failed with `expected </tr<%>`.

The row later closes its ERB loop with trim syntax:

```erb
  <td><%= friendly_date_ago(note.updated_at) %></td>
</tr>
<% end -%>
</table>
```

The initial parser error was:

```text
mismatched HTML close tag `</tr>`, expected `</tr<%>` at line 61, column 5
```

Fix:

- Treat `<` as a tag-name boundary when tokenizing HTML tags.
- Preserve the full raw opening tag so the formatter can continue to keep
  conditional-attribute ERB tags unchanged.
- Add regression coverage for `<tr<% if ... %> ... <% end %>>` followed by
  `<% end -%>`.

Verification:

```bash
cargo run --quiet -- --check external/rails-repos/openstreetmap-website/app/views/notes/index.html.erb
```

now reaches the normal format-check result:

```text
external/rails-repos/openstreetmap-website/app/views/notes/index.html.erb: file is not formatted.
```

## Formatter risks found

These are changes where formatted output may preserve Ruby meaning but still
change rendered HTML whitespace or visual layout. The initial audit found
several inline-sensitive cases. Current formatter work has since moved these
cases toward safer output.

### Inline ERB output inside text-like elements

Initial risk from `app/views/changeset_comments/feeds/_comment.html.erb`:

```diff
-<h2><%= t ".comment",
-          :author => comment.author.display_name,
-          :changeset_id => comment.changeset.id.to_s %></h2>
+<h2>
+  <%=
+    t ".comment",
+    :author => comment.author.display_name,
+    :changeset_id => comment.changeset.id.to_s
+  %>
+</h2>
```

Status: mitigated for foldable Ruby calls. The formatter now keeps the HTML
boundary inline and wraps the Ruby call inside the ERB output tag when it can do
so safely:

```erb
<h2><%= t(
  ".comment",
  author: comment.author.display_name,
  changeset_id: comment.changeset.id.to_s
) %></h2>
```

If the Ruby expression is not safely foldable, such as old-style hashrocket
options, the output is preserved inline instead of being expanded into
standalone ERB delimiters:

```erb
<h2><%= t ".comment", :author => user.name, :changeset_id => id %></h2>
```

This avoids adding literal whitespace around heading content.

### Text and helper output adjacency

Initial risk from `app/views/browse/_common_details.html.erb`: translated text
and a following `link_to` helper could be separated by ERB delimiter newlines,
adding literal whitespace around an inline phrase.

Status: mitigated. The formatter now treats text/helper adjacency as
inline-sensitive and keeps the surrounding text boundary intact while still
wrapping a foldable helper call:

```erb
<p>Location <%= link_to(
  t(".coordinates_html", latitude: lat, longitude: lon),
  root_path(anchor: "map=18/35/139")
) %></p>
```

If the helper is not foldable, it stays inline.

### Inline heading prefix adjacency

Initial risk from `app/views/browse/_common_details.html.erb`: a heading label
was followed by a literal `#` and another ERB output. Expanding either side into
standalone multiline ERB can change spacing around the `#` prefix.

Status: mitigated. Non-foldable expressions stay inline, and foldable helpers
after a text or symbol boundary wrap internally:

```erb
<h4><%= if redacted? then t("redacted") else t("version") end %>#<%= link_to(
  version,
  path
) %></h4>
```

The important safety property is that the `#<%=` boundary remains adjacent.

### Modifier output expanded into multiline ERB

Initial risk from `app/views/accounts/terms/show.html.erb`:

```diff
-<%= hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil? %>
+<%=
+  hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil?
+%>
```

Status: fixed. Output ERB tags whose Ruby body is a modifier expression
(`expr if cond`, `expr unless cond`) are no longer expanded into standalone ERB
delimiters:

```erb
<%= hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil? %>
```

This is intentionally conservative because modifier expressions are commonly
used inline.

### Long HTML tag wrapping

Example from `app/views/changeset_comments/feeds/_comment.html.erb`:

```diff
-<div class="changeset-comment-description" style="font-size: smaller; color: #999999"><%= t ".commented_at_by_html", :when => friendly_date_ago(comment.created_at), :user => comment.author.display_name %></div>
+<div
+  class="changeset-comment-description"
+  style="font-size: smaller; color: #999999"
+><%= t ".commented_at_by_html", :when => friendly_date_ago(comment.created_at), :user => comment.author.display_name %></div>
```

Status: verified and covered by regression tests. This is the preferred safety
shape for long opening tags with inline children: attributes wrap, but the
closing `>` remains adjacent to the inline child.

The same rule applies to text children:

```erb
<span
  class="very very very very very long class name"
  data-controller="x"
>Text</span>
```

and ERB children:

```erb
<div
  class="very very very very very long class name"
  data-controller="x"
><%= helper %></div>
```

Future tag wrapping should avoid turning these into:

```erb
>
  <%= ... %>
```

unless the child is known to be block-safe.

## Likely-safe changes

The audit also found many changes that look safe and useful:

- Indenting nested HTML under ERB control flow.
- Splitting long block-level opening tags by attribute.
- Normalizing extra spaces inside HTML tags.
- Wrapping standalone helper calls in block-level form containers.

These should remain supported, but regression tests should continue to
distinguish them from inline-sensitive cases.

## Remaining formatter safety candidates

The second diff audit did not find parser failures, but it still surfaced
formatter changes worth prioritizing before treating real-world formatting as
low-risk.

### Empty inline icon/button tags expanded to multiline content

Examples:

```diff
-      <i class="bi bi-chat<%= num_comments.zero? ? "" : "-fill" %> small" aria-hidden="true"></i>
+      <i
+        class="bi bi-chat<%= num_comments.zero? ? "" : "-fill" %> small"
+        aria-hidden="true"
+      ></i>
```

```diff
-        <button class="dropdown-toggle nav-link btn btn-outline-secondary px-1 py-0 border-0 bg-body text-secondary" type="button" data-bs-toggle="dropdown"><%= t ".more" %></button>
+        <button
+          class="dropdown-toggle nav-link btn btn-outline-secondary px-1 py-0 border-0 bg-body text-secondary"
+          type="button"
+          data-bs-toggle="dropdown"
+        ><%= t ".more" %></button>
```

Status: fixed. The formatter now keeps empty element close tags adjacent to the
wrapped opening tag. This avoids introducing a whitespace text child between an
empty inline element's open and close tags.

Preferred shapes:

```erb
<i
  class="very very very very very long icon name"
  aria-hidden="true"
></i>
```

```erb
<button
  class="button button--primary button--wide"
  data-controller="confirm"
  type="button"
>Save</button>
```

### Old-style hashrocket helper continuation indentation

Example:

```diff
 <h2><%= t ".comment",
-          :author => comment.author.display_name,
-          :changeset_id => comment.changeset.id.to_s %></h2>
+:author => comment.author.display_name,
+:changeset_id => comment.changeset.id.to_s %></h2>
```

This does not appear to change rendered HTML because the ERB output boundary
stays inline. It is still a readability regression. It also shows that
hashrocket-style helper calls are being preserved, but their continuation
indentation is not.

Status: fixed. Existing multiline ERB output inside inline-sensitive HTML is
preserved when the expression is not safely foldable. Single-line old-style
hash option helpers can now be folded when the Ruby AST recognizes them as safe
call arguments. Multiline old-style helpers keep their original continuation
indentation intact:

```erb
<h2><%= t ".comment",
          :author => comment.author.display_name,
          :changeset_id => comment.changeset.id.to_s %></h2>
```

Single-line old-style option helpers now fold while preserving the `=>` syntax:

```erb
<h2><%= t(
  ".comment",
  :author => comment.author.display_name,
  :changeset_id => comment.changeset.id.to_s
) %></h2>
```

Preservation still applies to multiline output helpers and ERB block openers
that use old-style hash arguments and cannot be safely normalized:

```erb
<%= inline_svg_tag "auth_providers/example.svg",
                   :class => "rounded-1 text-body-emphasis",
                   :title => t("application.auth_providers.example.alt"),
                   :size => "36" %>
```

```erb
<%= link_to auth_path(:provider => provider),
            :method => :post,
            :class => ["auth_button btn btn-outline-secondary border p-2", { "px-4 d-flex gap-3 justify-content-center align-items-center" => preferred }],
            :title => t("application.auth_providers.#{provider}.title") do %>
  <span>Sign in</span>
<% end %>
```

### Punctuation adjacency around multiline ERB output

Initial risk from `app/views/browse/_way.html.erb`:

```diff
-                (<%= t ".also_part_of_html",
-                       :count => related_ways.size,
-                       :related_ways => to_sentence(related_ways.map { |w| element_icon("way", w) + icon_connector + element_single_current_link("way", w) }) %>)
+                (
+                <%= t ".also_part_of_html",
+                :count => related_ways.size,
+                :related_ways => to_sentence(related_ways.map { |w| element_icon("way", w) + icon_connector + element_single_current_link("way", w) }) %>
+                )
```

Status: fixed. A multiline, non-foldable ERB output tag that shares a source
line with inline text or punctuation is preserved with that boundary. This
avoids inserting literal whitespace around punctuation such as parentheses.

### Standalone long ERB control tags expanded to delimiter lines

Example:

```diff
-<% if Settings.status != "api_offline" && Settings.status != "database_offline" %>
+<%
+  if Settings.status != "api_offline" && Settings.status != "database_offline"
+%>
```

This is usually safe for block-level control flow and remains acceptable. It is
not the same risk class as inline ERB output. It should remain covered by tests
so future changes do not apply this shape inside inline text boundaries.

## Covered formatter fixtures

The following audit-driven formatter cases are now covered or intentionally
preserved:

```erb
<h2><%= t(
  ".comment",
  :author => user.name,
  :id => id
) %></h2>
```

```erb
<h4><%= if redacted? then t("redacted") else t("version") end %>#<%= link_to(
  version,
  path
) %></h4>
```

```erb
<p>Location <%= link_to(
  t(".coordinates_html", latitude: lat, longitude: lon),
  root_path(anchor: "map=18/35/139")
) %></p>
```

```erb
<%= hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil? %>
```

```erb
<span
  class="very very very very very long class name"
  data-controller="x"
>Text</span>
```

```erb
<i
  class="very very very very very long icon name"
  aria-hidden="true"
></i>
```

```erb
<h2><%= t ".comment",
          :author => user.name,
          :id => id %></h2>
```

```erb
(<%= t ".also_part_of_html",
       :count => related_ways.size,
       :related_ways => to_sentence(related_ways) %>)
```

## Remaining proposed fixtures

No parser fixtures from the initial OpenStreetMap audit remain unimplemented.
Future parser audit findings should move into unit tests as they are discovered.
For formatter safety, empty inline elements, hashrocket continuation
preservation, and punctuation adjacency around non-foldable multiline ERB
output are now covered. Future fixtures should come from a fresh external diff
review.

## Summary

`erbfmt 0.3.0-dev` can process the OpenStreetMap Rails view corpus without the
two parser failures found in the initial audit. The original
audit showed two categories of risk:

1. Parser gaps around multiline ERB block openers and conditional attributes
   after HTML tag names.
2. Formatter whitespace changes around inline ERB output.

The parser gaps from this audit have been fixed. The full `app/views` check now
continues through the corpus and reports only formatted / not formatted files.

The second category has largely been mitigated by preserving inline boundaries,
wrapping only foldable Ruby calls inside ERB delimiters, and leaving
non-foldable expressions inline. The `</tr<%>` parse error from conditional
attributes has also been fixed. The main remaining audit follow-up is to rerun
the external diff review periodically and promote any new safety concerns into
fixtures. The latest diff audit after legacy hash option folding changed 160
files and 6839 unified diff lines; the increase is expected because more
old-style Rails helpers can now be folded. The remaining sampled changes are
mostly block indentation, long opening tag wrapping, and foldable ERB helper
wrapping rather than known inline-whitespace risks.
