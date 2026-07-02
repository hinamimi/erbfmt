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

The parse errors were:

```text
Error: failed to parse `app/views/layers_panes/show.html.erb`: unclosed HTML tag `<div class="overlay-layers p-3">` at line 20, column 3
Error: failed to parse `app/views/notes/index.html.erb`: mismatched HTML close tag `</tr>`, expected `</tr<%>` at line 61, column 5
```

Running `--write` on a temporary copy wrote 239 files, failed on the same 2
files, and changed 166 files.

## Parser issues found

### Multiline ERB block opener with Ruby setup

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

Safety impact:

- Formatter does not run for the file.
- This is safer than rewriting incorrectly, but the diagnostic points at the
  outer HTML tag instead of the unsupported ERB block shape.

Follow-up:

- Support multiline ERB block openers whose tag body contains setup statements.
- If that is too broad, emit a clearer unsupported-ERB diagnostic without
  blaming the surrounding HTML.

### Trim marker confused with an HTML close tag

`app/views/notes/index.html.erb` contains a table row loop ending with
`<% end -%>`.

```erb
  <td><%= friendly_date_ago(note.updated_at) %></td>
</tr>
<% end -%>
</table>
```

The parser reports:

```text
mismatched HTML close tag `</tr>`, expected `</tr<%>` at line 61, column 5
```

This looks like the ERB trim marker or ERB boundary leaked into the HTML tag
stack.

Safety impact:

- Formatter does not run for the file.
- The error message is misleading and makes the actual source hard to identify.

Follow-up:

- Normalize `<% ... -%>` boundaries before HTML stack comparison.
- Add a regression fixture for table row loops using trim markers.

## Formatter risks found

These are changes where formatted output may preserve Ruby meaning but still
change rendered HTML whitespace or visual layout.

### Inline ERB output expanded inside text-like elements

Example from `app/views/changeset_comments/feeds/_comment.html.erb`:

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

The Ruby output is equivalent, but the formatter introduces literal whitespace
around the heading content. Browsers usually collapse this whitespace, but it can
still matter for inline layout, exact text rendering, copy/paste behavior, and
CSS that depends on whitespace-sensitive contexts.

Recommendation:

- If an element was originally a single-line text-like wrapper around one ERB
  output, keep it inline even when the ERB expression is long.
- Prefer preserving `<h2><%= ... %></h2>` over aggressive line wrapping.

### Text and helper output separated by inserted newlines

Example from `app/views/browse/_common_details.html.erb`:

```diff
 <div>
   <%= t "browse.location" %>
-  <%= link_to(t(".coordinates_html",
-                :latitude => tag.span(number_with_delimiter(common_details.lat), :class => "latitude"),
-                :longitude => tag.span(number_with_delimiter(common_details.lon), :class => "longitude")),
-              root_path(:anchor => "map=18/#{common_details.lat}/#{common_details.lon}")) %>
+  <%=
+    link_to(t(".coordinates_html",
+      :latitude => tag.span(number_with_delimiter(common_details.lat), :class => "latitude"),
+      :longitude => tag.span(number_with_delimiter(common_details.lon), :class => "longitude")),
+    root_path(:anchor => "map=18/#{common_details.lat}/#{common_details.lon}"))
+  %>
 </div>
```

The formatter moves the link helper to a multiline ERB output tag. In a block
context this is often acceptable, but it still adds literal whitespace around an
inline link after translated text.

Recommendation:

- Treat adjacent text/helper output inside the same parent as inline-sensitive.
- Avoid expanding the ERB tag if the output participates in a sentence or inline
  phrase.

### Inline heading prefix separated from ERB output

Example from `app/views/browse/_common_details.html.erb`:

```diff
 <h4>
-  <%= if common_details.redacted?
-        t "browse.redacted_version"
-      else
-        t "browse.version"
-      end %>
+  <%=
+    if common_details.redacted?
+      t "browse.redacted_version"
+    else
+      t "browse.version"
+    end
+  %>
   #<%= link_to_unless_current common_details.version, ... %>
 </h4>
```

The inserted newlines can affect the spacing before the `#` prefix. This is a
stronger visual-risk signal than a standalone block-level helper call.

Recommendation:

- Preserve inline adjacency between ERB output and following text or symbols.
- Add a fixture for `<h4><%= ... %>#<%= ... %></h4>` and similar heading labels.

### Modifier output expanded into multiline ERB

Example from `app/views/accounts/terms/show.html.erb`:

```diff
-<%= hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil? %>
+<%=
+  hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil?
+%>
```

This is probably safe inside the current block, but modifier expressions are
commonly used inline. Expanding them can add whitespace even when the expression
does not output anything.

Recommendation:

- Do not wrap output ERB tags whose Ruby body is a modifier expression
  (`expr if cond`, `expr unless cond`) unless the surrounding HTML context is
  known to be block-safe.

### Long HTML tag wrapping appears mostly safe here

Example from `app/views/changeset_comments/feeds/_comment.html.erb`:

```diff
-<div class="changeset-comment-description" style="font-size: smaller; color: #999999"><%= t ".commented_at_by_html", :when => friendly_date_ago(comment.created_at), :user => comment.author.display_name %></div>
+<div
+  class="changeset-comment-description"
+  style="font-size: smaller; color: #999999"
+><%= t ".commented_at_by_html", :when => friendly_date_ago(comment.created_at), :user => comment.author.display_name %></div>
```

This preserves the `>` and `<%=` boundary on the same line, so it does not insert
whitespace before the inline content. This is the desired safety shape for long
opening tags with inline children.

Recommendation:

- Keep this behavior.
- Ensure future tag wrapping does not turn this into:

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

These should remain supported, but regression tests should distinguish them from
inline-sensitive cases.

## Proposed fixtures

Add formatter fixtures for:

```erb
<h2><%= t ".comment", :author => user.name, :id => id %></h2>
```

```erb
<h4><%= if redacted? then t("redacted") else t("version") end %>#<%= link_to version, path %></h4>
```

```erb
<div><%= t "browse.location" %> <%= link_to(t(".coordinates_html", :latitude => lat, :longitude => lon), root_path) %></div>
```

```erb
<%= hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil? %>
```

```erb
<% items.each do |item|
     options = { :class => "form-check" }
     options[:title] = item.title if item.title? %>
  <%= tag.div(**options) do %>
    <%= item.name %>
  <% end %>
<% end %>
```

```erb
<tr>
  <td><%= value %></td>
</tr>
<% end -%>
```

## Summary

`erbfmt 0.3.0-dev` can process most OpenStreetMap Rails views, but the audit
shows two immediate safety priorities:

1. Improve parser handling for multiline ERB block openers and trim markers.
2. Make ERB wrapping more conservative when the output is adjacent to inline
   text, symbols, or single-line text-like elements.

The second item is the main visual-regression risk. Even when Ruby meaning is
unchanged, inserted ERB tag newlines can alter rendered whitespace.
