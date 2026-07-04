use super::*;
use crate::{
    lexer::{tokenize, tokenize_with_spans},
    mixed_parser::{parse, parse_spanned},
};

fn format(input: &str) -> String {
    let tokens = tokenize(input).unwrap();
    let document = parse(&tokens).unwrap();

    format_document(&document)
}

fn format_without_html_indent(input: &str) -> String {
    let tokens = tokenize(input).unwrap();
    let document = parse(&tokens).unwrap();

    format_document_with_options(
        &document,
        FormatOptions {
            indent_html: false,
            ..FormatOptions::default()
        },
    )
}

fn format_with_options(input: &str, options: FormatOptions) -> String {
    let tokens = tokenize(input).unwrap();
    let document = parse(&tokens).unwrap();

    format_document_with_options(&document, options)
}

fn format_source(input: &str) -> String {
    format_source_with_options(input, FormatOptions::default())
}

fn format_source_with_options(input: &str, options: FormatOptions) -> String {
    let tokens = tokenize_with_spans(input).unwrap();
    let document = parse_spanned(&tokens).unwrap();

    format_document_with_source(&document, input, options)
}

#[test]
fn formats_plain_html_lines() {
    assert_eq!(
        format("<div>\n<p>Hello</p>\n</div>\n"),
        "<div>\n  <p>Hello</p>\n</div>\n"
    );
}

#[test]
fn normalizes_html_tag_spacing() {
    assert_eq!(
        format(r#"<div  class="test" >test</div >"#),
        "<div class=\"test\">test</div>\n"
    );
    assert_eq!(
        format(r#"<p><span  class="test" >test</span ></p>"#),
        "<p><span class=\"test\">test</span></p>\n"
    );
}

#[test]
fn normalizes_html_tag_spacing_when_expanding() {
    assert_eq!(
        format("<div  class=\"test\" >\n<p>test</p>\n</div >"),
        "<div class=\"test\">\n  <p>test</p>\n</div>\n"
    );
}

#[test]
fn preserves_standalone_erb_attributes_in_html_tags() {
    assert_eq!(
        format(r#"<details <%= "open" if expanded? %> class=""></details>"#),
        "<details <%= \"open\" if expanded? %> class=\"\"></details>\n"
    );
}

#[test]
fn wraps_standalone_erb_attributes_in_long_html_tags() {
    assert_eq!(
        format_with_options(
            r#"<details <%= "open" unless request.user_agent&.match?(/Mobile|Android|iPhone|iPad/) %> class=""></details>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<details\n  <%= \"open\" unless request.user_agent&.match?(/Mobile|Android|iPhone|iPad/) %>\n  class=\"\"\n></details>\n"
    );
}

#[test]
fn preserves_inline_link_to_inside_list_items() {
    let input = r#"<li class=""><%= link_to "home", root_path, class: "" %></li>"#;
    let expected = "<li class=\"\"><%= link_to \"home\", root_path, class: \"\" %></li>\n";

    assert_eq!(format(input), expected);
    assert_eq!(
        format_source_with_options(
            &format!("{input}\n"),
            FormatOptions {
                line_width: 30,
                ..FormatOptions::default()
            }
        ),
        "<li class=\"\"><%= link_to(\n  \"home\",\n  root_path,\n  class: \"\"\n) %></li>\n"
    );
}

#[test]
fn preserves_multiline_erb_code_tags_with_comments() {
    let input = "<%\n  # comments\n  x = test\n  if condition\n    call\n  end\n%>\n";

    assert_eq!(format_source(input), input);
    assert_eq!(
        format_source_with_options(
            input,
            FormatOptions {
                line_width: 30,
                ..FormatOptions::default()
            }
        ),
        input
    );
}

#[test]
fn preserves_unquoted_attribute_trailing_slash_values() {
    assert_eq!(
        format("<img src=/assets/logo/>\n"),
        "<img src=/assets/logo/>\n"
    );
    assert_eq!(
        format("<div data-url=/assets/logo/>Logo</div>\n"),
        "<div data-url=/assets/logo/>Logo</div>\n"
    );
}

#[test]
fn normalizes_quoted_attribute_self_closing_tags() {
    assert_eq!(
        format(r#"<custom-input name="profile[display_name]"/>"#),
        "<custom-input name=\"profile[display_name]\" />\n"
    );
}

#[test]
fn preserves_single_intentional_blank_lines() {
    assert_eq!(
        format("<section>\n<h1>Title</h1>\n\n<p>Body</p>\n</section>\n"),
        "<section>\n  <h1>Title</h1>\n\n  <p>Body</p>\n</section>\n"
    );
}

#[test]
fn collapses_multiple_blank_lines_to_one() {
    assert_eq!(
        format("<section>\n<h1>Title</h1>\n\n\n<p>Body</p>\n</section>\n"),
        "<section>\n  <h1>Title</h1>\n\n  <p>Body</p>\n</section>\n"
    );
}

#[test]
fn preserves_inline_erb_output() {
    assert_eq!(
        format("<h1><%= page_title %></h1>\n<p>Hello, <%= user.name %></p>\n"),
        "<h1><%= page_title %></h1>\n<p>Hello, <%= user.name %></p>\n"
    );
}

#[test]
fn preserves_text_adjacent_to_inline_html_inside_erb_blocks() {
    let code_block = "<% link_to(user_path(user)) do %>\n<i class=\"icon\"></i>test\n<% end %>\n";
    let output_block =
        "<%= link_to(user_path(user)) do %>\n<i class=\"icon\"></i>test\n<% end %>\n";
    let formatted = "<% link_to(user_path(user)) do %>\n  <i class=\"icon\"></i>test\n<% end %>\n";

    assert_eq!(format(code_block), formatted);
    assert_eq!(format(formatted), formatted);
    assert_eq!(
        format(output_block),
        "<%= link_to(user_path(user)) do %>\n  <i class=\"icon\"></i>test\n<% end %>\n"
    );

    assert_eq!(
        format_with_options(
            "<p><i class=\"long-icon-name\"></i>test</p>\n",
            FormatOptions {
                line_width: 20,
                ..FormatOptions::default()
            }
        ),
        "<p><i class=\"long-icon-name\"></i>test</p>\n"
    );
}

#[test]
fn preserves_inline_html_adjacency_in_both_directions_across_lines() {
    let input = "<i class=\"icon\"></i>テスト\nテスト<i class=\"icon\"></i>\n";

    assert_eq!(format(input), input);

    assert_eq!(
        format(&format!("<% if visible? %>\n{input}<% end %>\n")),
        "<% if visible? %>\n  <i class=\"icon\"></i>テスト\n  テスト<i class=\"icon\"></i>\n<% end %>\n"
    );

    let long_input =
        "<i class=\"long-icon-name\"></i>テスト\nテスト<i class=\"long-icon-name\"></i>\n";
    assert_eq!(
        format_with_options(
            long_input,
            FormatOptions {
                line_width: 20,
                ..FormatOptions::default()
            }
        ),
        long_input
    );
}

#[test]
fn preserves_whitespace_boundaries_around_inline_html() {
    let input = "<p>Hello <strong>world</strong>!</p>\n<span> padded </span>\n<i></i>\ntext\n";

    assert_eq!(format(input), input);
}

#[test]
fn preserves_outer_whitespace_around_root_inline_html_fragments() {
    let input = " Hello <strong>world</strong> \n";

    assert_eq!(format_source(input), " Hello <strong>world</strong>\n");
}

#[test]
fn preserves_inline_boundaries_around_custom_elements() {
    let options = FormatOptions {
        line_width: 36,
        ..FormatOptions::default()
    };
    let input = "<a class=\"button button--primary button--wide\"><ui-icon name=\"check\"></ui-icon>Done</a>\n";
    let expected = "<a\n  class=\"button button--primary button--wide\"\n><ui-icon name=\"check\"></ui-icon>Done</a>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn preserves_source_boundaries_between_opening_tags_and_inline_children() {
    let options = FormatOptions {
        line_width: 36,
        ..FormatOptions::default()
    };
    let input = "<a class=\"button button--primary\" href=\"/profile\">Profile</a>\n";
    let expected = "<a\n  class=\"button button--primary\"\n  href=\"/profile\"\n>Profile</a>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn preserves_boundaries_when_wrapping_long_tags_with_inline_text_children() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = "<span class=\"very very very very very long class name\" data-controller=\"x\">Text</span>\n";
    let expected = "<span\n  class=\"very very very very very long class name\"\n  data-controller=\"x\"\n>Text</span>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn preserves_boundaries_when_wrapping_long_tags_with_inline_erb_children() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = "<div class=\"very very very very very long class name\" data-controller=\"x\"><%= helper %></div>\n";
    let expected = "<div\n  class=\"very very very very very long class name\"\n  data-controller=\"x\"\n><%= helper %></div>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn preserves_empty_element_boundaries_when_wrapping_long_tags() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = "<i class=\"very very very very very long icon name\" aria-hidden=\"true\"></i>\n";
    let expected =
        "<i\n  class=\"very very very very very long icon name\"\n  aria-hidden=\"true\"\n></i>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn preserves_inline_child_boundaries_when_wrapping_long_button_tags() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = "<button class=\"button button--primary button--wide\" data-controller=\"confirm\" type=\"button\">Save</button>\n";
    let expected = "<button\n  class=\"button button--primary button--wide\"\n  data-controller=\"confirm\"\n  type=\"button\"\n>Save</button>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn preserves_source_boundaries_between_closing_tags_and_inline_children() {
    let input = "<p>Lead\n<strong>Body</strong>\nTail</p>\n";
    let expected = "<p>Lead\n  <strong>Body</strong>\n  Tail</p>\n";

    assert_eq!(format_source(input), expected);
    assert_eq!(format_source(expected), expected);
}

#[test]
fn preserves_text_adjacency_across_inline_comments() {
    let input = "<p>first<!-- separator -->second<%# note %>third</p>\n";

    assert_eq!(format(input), input);
}

#[test]
fn preserves_erb_comment_markers() {
    assert_eq!(
        format_source("<%# generated note %>\n"),
        "<%# generated note %>\n"
    );
}

#[test]
fn preserves_supported_erb_marker_variants() {
    assert_eq!(
        format_source("<p><%== raw_html %></p>\n"),
        "<p><%== raw_html %></p>\n"
    );
    assert_eq!(
        format_source("<% foo -%>\n<p>Next</p>\n"),
        "<% foo -%>\n<p>Next</p>\n"
    );
    assert_eq!(
        format_source("<%- if visible? -%>\n<span>Visible</span>\n<%- end -%>\n"),
        "<%- if visible? -%>\n  <span>Visible</span>\n<%- end -%>\n"
    );
}

#[test]
fn preserves_adjacent_erb_outputs_on_one_line() {
    assert_eq!(
        format("<%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n"),
        "<%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n"
    );
}

#[test]
fn preserves_adjacent_erb_outputs_inside_blocks() {
    assert_eq!(
        format(
            "<% if form %>\n<%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n<% end %>\n"
        ),
        "<% if form %>\n  <%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n<% end %>\n"
    );
}

#[test]
fn preserves_single_line_erb_blocks_inline() {
    let options = FormatOptions {
        line_width: 24,
        ..FormatOptions::default()
    };
    let code_block = "<% if visible? %><span>Visible</span><% end %>\n";
    let output_block = "<%= link_to profile_path do %><i class=\"icon\"></i>Profile<% end %>\n";

    assert_eq!(format_source_with_options(code_block, options), code_block);
    assert_eq!(
        format_source_with_options(output_block, options),
        output_block
    );
}

#[test]
fn preserves_inline_text_boundaries_around_multiline_erb_blocks() {
    let input = "Hello<% if user %>\n<span><%= user.name %></span>\n<% end %>!\n";
    let expected = "Hello<% if user %>\n  <span><%= user.name %></span>\n<% end %>!\n";

    assert_eq!(format_source(input), expected);
    assert_eq!(format_source(expected), expected);

    let paragraph = "<p>Hello<% if user %>\n<span>Admin</span>\n<% end %>!</p>\n";
    assert_eq!(format_source(paragraph), paragraph);
}

#[test]
fn preserves_formatter_ignored_html_and_erb_nodes() {
    assert_eq!(
        format_source(formatter_ignore_fixture()),
        formatter_ignore_fixture()
    );
}

#[test]
fn preserves_formatter_ignored_erb_block_subtrees() {
    let input = "<!-- erbfmt-ignore format: generated block -->\n<% if user %>\n<p   class=\"legacy\">Keep   spacing</p>\n<% else %>\n<p>Also  keep</p>\n<% end %>\n";

    assert_eq!(format_source(input), input);
}

#[test]
fn preserves_formatter_ignored_nodes_with_combined_directives() {
    let input = "<!-- erbfmt-ignore all: generated markup -->\n<div   class=\"generated\"><center>Keep this</center></div>\n";

    assert_eq!(format_source(input), input);
}

#[test]
fn formats_nodes_surrounding_formatter_ignored_subtrees() {
    let input = "<section>\n<!-- erbfmt-ignore format: legacy -->\n    <div   class=\"legacy\">Keep   spacing</div>\n<p>Normal</p>\n</section>\n";

    assert_eq!(
        format_source(input),
        "<section>\n  <!-- erbfmt-ignore format: legacy -->\n    <div   class=\"legacy\">Keep   spacing</div>\n  <p>Normal</p>\n</section>\n"
    );
}

#[test]
fn falls_back_when_formatter_ignore_target_is_not_on_the_next_line() {
    let input = "<!-- erbfmt-ignore format: separated -->\n\n<article class=\"card\" data-controller=\"profile\" aria-label=\"Profile card\"></article>\n";

    assert_eq!(
        format_source_with_options(
            input,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<!-- erbfmt-ignore format: separated -->\n\n<article\n  class=\"card\"\n  data-controller=\"profile\"\n  aria-label=\"Profile card\"\n></article>\n"
    );
}

#[test]
fn formatter_ignore_is_idempotent() {
    let once = format_source(formatter_ignore_fixture());
    let twice = format_source(&once);

    assert_eq!(twice, once);
}

#[test]
fn formatter_ignore_preserves_source_line_endings() {
    let input = "<section>\r\n<!-- erbfmt-ignore format: legacy -->\r\n    <div   class=\"legacy\">Keep   spacing</div>\r\n<p>Normal</p>\r\n</section>\r\n";

    assert_eq!(
        format_source_with_options(
            input,
            FormatOptions {
                line_ending: LineEnding::Lf,
                ..FormatOptions::default()
            }
        ),
        "<section>\n  <!-- erbfmt-ignore format: legacy -->\n    <div   class=\"legacy\">Keep   spacing</div>\r\n  <p>Normal</p>\n</section>\n"
    );
}

#[test]
fn preserves_preformatted_html_content() {
    assert_eq!(
        format("<div>\n<pre>\n  line one\n    line two\n</pre>\n</div>\n"),
        "<div>\n  <pre>\n  line one\n    line two\n</pre>\n</div>\n"
    );
}

#[test]
fn preserves_inline_preformatted_html_content() {
    assert_eq!(
        format("<pre>  line one\n    line two</pre>\n"),
        "<pre>  line one\n    line two</pre>\n"
    );
}

#[test]
fn preserves_textarea_content() {
    assert_eq!(
        format("<form>\n<textarea>\n  keep me\n</textarea>\n</form>\n"),
        "<form>\n  <textarea>\n  keep me\n</textarea>\n</form>\n"
    );
}

#[test]
fn preserves_svg_math_and_contenteditable_subtrees() {
    let input = "<section>\n<svg viewBox=\"0 0 10 10\">\n  <path   d=\"M0 0L10 10\"></path>\n</svg>\n<math><mi>x</mi>  <mo>=</mo><mn>1</mn></math>\n<div contenteditable=\"true\"><p> keep  spacing</p></div>\n<div style=\"white-space: pre-line\"><span>A</span>\n    B</div>\n</section>\n";

    assert_eq!(
        format(input),
        "<section>\n  <svg viewBox=\"0 0 10 10\">\n  <path   d=\"M0 0L10 10\"></path>\n</svg>\n  <math><mi>x</mi>  <mo>=</mo><mn>1</mn></math>\n  <div contenteditable=\"true\"><p> keep  spacing</p></div>\n  <div style=\"white-space: pre-line\"><span>A</span>\n    B</div>\n</section>\n"
    );
}

#[test]
fn wraps_format_sensitive_opening_tags_without_touching_content() {
    let options = FormatOptions {
        line_width: 42,
        ..FormatOptions::default()
    };

    assert_eq!(
        format_source_with_options(
            "<svg class=\"icon icon--large\" viewBox=\"0 0 10 10\"><text>Hi</text></svg>\n",
            options
        ),
        "<svg\n  class=\"icon icon--large\"\n  viewBox=\"0 0 10 10\"\n><text>Hi</text></svg>\n"
    );

    assert_eq!(
        format_source_with_options(
            "<div class=\"editable editable--wide\" contenteditable=\"true\"><p> keep  spacing</p></div>\n",
            options
        ),
        "<div\n  class=\"editable editable--wide\"\n  contenteditable=\"true\"\n><p> keep  spacing</p></div>\n"
    );
}

#[test]
fn preserves_template_and_noscript_subtrees() {
    let input = "<section>\n<template><div   class=\"legacy\">Keep  spacing</div></template>\n<noscript><p> keep  spacing</p></noscript>\n</section>\n";

    assert_eq!(
        format(input),
        "<section>\n  <template><div   class=\"legacy\">Keep  spacing</div></template>\n  <noscript><p> keep  spacing</p></noscript>\n</section>\n"
    );
}

#[test]
fn preserves_script_and_style_content() {
    assert_eq!(
        format(
            "<script>\n  console.log(\"hello\");\n</script>\n<style>\n  body { color: red; }\n</style>\n"
        ),
        "<script>\n  console.log(\"hello\");\n</script>\n<style>\n  body { color: red; }\n</style>\n"
    );
}

#[test]
fn preserves_javascript_comparisons_inside_script_tags() {
    let input = "<script type=\"text/javascript\">\n(function(){\n  console.log(\"test\");\n  if(x < 10){\n    console.log(\"small\");\n  }\n  if (y > 10) {\n    console.log(\"large\");\n  }\n})()\n</script>\n";

    assert_eq!(format_source(input), input);
}

#[test]
fn preserves_erb_spacing_inside_format_sensitive_subtrees() {
    assert_eq!(
        format_source("<pre>  A <%=   value   %> B  </pre>\n"),
        "<pre>  A <%=   value   %> B  </pre>\n"
    );
    assert_eq!(
        format_source("<script>const value = \"<%=   value   %>\";</script>\n"),
        "<script>const value = \"<%=   value   %>\";</script>\n"
    );
    assert_eq!(
        format_source("<template><span><%=   value   %></span></template>\n"),
        "<template><span><%=   value   %></span></template>\n"
    );
}

#[test]
fn indents_erb_block_children() {
    assert_eq!(
        format("<% if user %>\n<p>Hello</p>\n<% end %>\n"),
        "<% if user %>\n  <p>Hello</p>\n<% end %>\n"
    );
}

#[test]
fn indents_nested_erb_blocks() {
    assert_eq!(
        format(
            "<% if user %>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n"
        ),
        "<% if user %>\n  <ul>\n    <% Objects.map do |obj| %>\n      <li><%= obj.name %></li>\n    <% end %>\n  </ul>\n<% end %>\n"
    );
}

#[test]
fn can_disable_html_indentation() {
    assert_eq!(
        format_without_html_indent(
            "<% if user %>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n"
        ),
        "<% if user %>\n  <ul>\n  <% Objects.map do |obj| %>\n    <li><%= obj.name %></li>\n  <% end %>\n  </ul>\n<% end %>\n"
    );
}

#[test]
fn preserves_conditional_erb_attribute_blocks_in_html_tags() {
    let input = "<table>\n<tr <% unless linked %>style=\"cursor: pointer\" onclick=\"console.log('test');\"<% end %>>\n<td>test</td>\n</tr>\n</table>\n";
    let expected = "<table>\n  <tr <% unless linked %>style=\"cursor: pointer\" onclick=\"console.log('test');\"<% end %>>\n    <td>test</td>\n  </tr>\n</table>\n";

    assert_eq!(format_source(input), expected);
}

#[test]
fn preserves_conditional_erb_attribute_blocks_even_when_long() {
    let input = "<tr <% if clickable? %>class=\"clickable cursor-pointer\" data-controller=\"row-link\" data-action=\"click->row-link#open\"<% end %>></tr>\n";

    assert_eq!(
        format_with_options(
            input,
            FormatOptions {
                line_width: 40,
                ..FormatOptions::default()
            }
        ),
        "<tr <% if clickable? %>class=\"clickable cursor-pointer\" data-controller=\"row-link\" data-action=\"click->row-link#open\"<% end %>></tr>\n"
    );
}

#[test]
fn can_configure_indent_width() {
    assert_eq!(
        format_with_options(
            "<div>\n<p>Hello</p>\n</div>\n",
            FormatOptions {
                indent_width: 4,
                ..FormatOptions::default()
            }
        ),
        "<div>\n    <p>Hello</p>\n</div>\n"
    );
}

#[test]
fn can_configure_tab_indentation() {
    assert_eq!(
        format_with_options(
            "<div>\n<p>Hello</p>\n</div>\n",
            FormatOptions {
                indent_style: IndentStyle::Tab,
                ..FormatOptions::default()
            }
        ),
        "<div>\n\t<p>Hello</p>\n</div>\n"
    );
}

#[test]
fn can_configure_line_ending_and_trailing_newline() {
    assert_eq!(
        format_with_options(
            "<div>\n<p>Hello</p>\n</div>\n",
            FormatOptions {
                line_ending: LineEnding::Crlf,
                trailing_newline: false,
                ..FormatOptions::default()
            }
        ),
        "<div>\r\n  <p>Hello</p>\r\n</div>"
    );
}

#[test]
fn wraps_long_html_opening_tags_by_attribute() {
    assert_eq!(
        format_with_options(
            r#"<article class="card" data-user-id="<%= user.id %>" aria-label="Current user profile"><p>Hello</p></article>"#,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<article\n  class=\"card\"\n  data-user-id=\"<%= user.id %>\"\n  aria-label=\"Current user profile\"\n>\n  <p>Hello</p>\n</article>\n"
    );
}

#[test]
fn wraps_erb_attribute_tags_at_line_width_boundary() {
    let input = r#"<button class="test <%= 'd-none' if test? %>" id=<%= button_id %> type="button">Save</button>
"#;
    let expected = "<button\n  class=\"test <%= 'd-none' if test? %>\"\n  id=<%= button_id %>\n  type=\"button\"\n>Save</button>\n";

    assert_eq!(format_source(input), expected);
    assert_eq!(format_source(expected), expected);
}

#[test]
fn wraps_long_inline_html_elements_by_attribute() {
    assert_eq!(
        format_with_options(
            r#"<div class="a-long-long-class-name" style="<%= long_logic %>"></div>"#,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<div\n  class=\"a-long-long-class-name\"\n  style=\"<%= long_logic %>\"\n></div>\n"
    );
}

#[test]
fn normalizes_safe_single_quoted_html_attributes_to_double_quotes() {
    assert_eq!(
        format("<div class='card' data-user-id='<%= user.id %>'></div>\n"),
        "<div class=\"card\" data-user-id=\"<%= user.id %>\"></div>\n"
    );
}

#[test]
fn preserves_single_quoted_html_attributes_when_double_quotes_are_inside_value() {
    assert_eq!(
        format(r#"<div data-json='{"key":"value"}' title='<%= t("profile.title") %>'></div>"#),
        "<div data-json='{\"key\":\"value\"}' title='<%= t(\"profile.title\") %>'></div>\n"
    );
}

#[test]
fn normalizes_single_quoted_html_attributes_when_wrapping_tags() {
    assert_eq!(
        format_with_options(
            r#"<div class='a-long-long-class-name' style='<%= long_logic %>'></div>"#,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<div\n  class=\"a-long-long-class-name\"\n  style=\"<%= long_logic %>\"\n></div>\n"
    );
}

#[test]
fn wraps_long_void_tags_by_attribute() {
    assert_eq!(
        format_with_options(
            r#"<img src="<%= avatar_url %>" alt="<%= user.name %>" data-controller="avatar-preview">"#,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<img\n  src=\"<%= avatar_url %>\"\n  alt=\"<%= user.name %>\"\n  data-controller=\"avatar-preview\"\n>\n"
    );
}

#[test]
fn wraps_long_self_closing_tags_with_marker_on_own_line() {
    assert_eq!(
        format_with_options(
            r#"<custom-input name="profile[display_name]" value="<%= user.display_name %>" data-controller="autosave" />"#,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<custom-input\n  name=\"profile[display_name]\"\n  value=\"<%= user.display_name %>\"\n  data-controller=\"autosave\"\n/>\n"
    );
}

#[test]
fn normalizes_existing_multiline_html_tags() {
    assert_eq!(
        format("<div\nclass=\"card\"\ndata-controller=\"profile\"\n>\n<p>Hello</p>\n</div>\n"),
        "<div\n  class=\"card\"\n  data-controller=\"profile\"\n>\n  <p>Hello</p>\n</div>\n"
    );
}

#[test]
fn normalizes_multiline_html_tags_with_erb_attributes() {
    assert_eq!(
        format_source(
            "<a\nhref=\"/users/<%= user.id %>\"\naria-label=\"<%= user.name %>\"\n>Profile</a>\n"
        ),
        "<a\n  href=\"/users/<%= user.id %>\"\n  aria-label=\"<%= user.name %>\"\n>Profile</a>\n"
    );
}

#[test]
fn wraps_long_erb_output_command_calls() {
    assert_eq!(
        format_with_options(
            r#"<%= link_to "Edit profile", edit_user_path(user), class: "button button--primary", data: { turbo_frame: "_top" } %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%= link_to(\n  \"Edit profile\",\n  edit_user_path(user),\n  class: \"button button--primary\",\n  data: { turbo_frame: \"_top\" }\n) %>\n"
    );
}

#[test]
fn wraps_long_erb_output_keyword_hash_arguments() {
    assert_eq!(
        format_with_options(
            r#"<%= render partial: "profile", locals: { current_user: current_user, account: account, selected_status: selected_status } %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%= render(\n  partial: \"profile\",\n  locals: {\n    current_user: current_user,\n    account: account,\n    selected_status: selected_status\n  }\n) %>\n"
    );
}

#[test]
fn wraps_long_erb_output_single_keyword_hash_argument() {
    assert_eq!(
        format_with_options(
            r#"<%= render locals: { current_user: current_user, account: account, selected_status: selected_status } %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%= render(\n  locals: {\n    current_user: current_user,\n    account: account,\n    selected_status: selected_status\n  }\n) %>\n"
    );
}

#[test]
fn preserves_long_erb_code_tags_when_arguments_are_not_safely_splittable() {
    assert_eq!(
        format_with_options(
            r#"<% cache ["profile-card", user.cache_key_with_version, current_user.cache_key_with_version] %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%\n  cache [\"profile-card\", user.cache_key_with_version, current_user.cache_key_with_version]\n%>\n"
    );
}

#[test]
fn preserves_heredoc_erb_tags() {
    let root = "<%\nmessage = <<TEXT\nhello\nTEXT\n%>\n";
    assert_eq!(format_source(root), root);

    assert_eq!(
        format_source("<div>\n<%\nmessage = <<TEXT\nhello\nTEXT\n%>\n</div>\n"),
        "<div>\n  <%\nmessage = <<TEXT\nhello\nTEXT\n%>\n</div>\n"
    );
}

#[test]
fn preserves_self_contained_multiline_erb_case_tags() {
    let input = "<% name = 1 %>\n\n<%\n  case name\n  when 1\n    x = \"1\"\n  when 2\n    x = \"2\"\n  end\n%>\n";

    assert_eq!(format_source(input), input);
}

#[test]
fn preserves_long_erb_block_opening_tags_when_they_are_control_flow() {
    let options = FormatOptions {
        line_width: 60,
        ..FormatOptions::default()
    };
    let input = "<% if current_user.admin? && feature_enabled?(:new_dashboard) && account.active? %>\n<p>Hello</p>\n<% end %>\n";
    let expected = "<%\n  if current_user.admin? && feature_enabled?(:new_dashboard) && account.active?\n%>\n  <p>Hello</p>\n<% end %>\n";

    assert_eq!(format_with_options(input, options), expected);
    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn preserves_multiline_erb_block_openers_with_setup_statements() {
    let input = r#"<% @overlay_layers.each do |layer|
     options = { :data => layer, :class => "form-check" }
     options[:title] = t(".tooltip") if layer[:max_area] %>
<span><%= layer[:name] %></span>
<% end %>
"#;
    let expected = r#"<% @overlay_layers.each do |layer|
     options = { :data => layer, :class => "form-check" }
     options[:title] = t(".tooltip") if layer[:max_area] %>
  <span><%= layer[:name] %></span>
<% end %>
"#;

    assert_eq!(format_source(input), expected);
    assert_eq!(format_source(expected), expected);
}

#[test]
fn wraps_long_erb_code_command_calls() {
    assert_eq!(
        format_with_options(
            r#"<% tag.div class: "card", data: { controller: "profile" }, aria: { label: "Profile" } %>"#,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<%\n  tag.div(\n    class: \"card\",\n    data: { controller: \"profile\" },\n    aria: { label: \"Profile\" }\n  )\n%>\n"
    );
}

#[test]
fn preserves_multistatement_erb_code_tags_when_wrapping() {
    let input = "<%\n  content_for :header, render(\"header\")\n  content_for :footer, render(\"footer\")\n%>\n";

    assert_eq!(
        format_source_with_options(
            input,
            FormatOptions {
                line_width: 40,
                ..FormatOptions::default()
            }
        ),
        input
    );
}

#[test]
fn wraps_long_erb_output_command_calls_with_do_blocks() {
    assert_eq!(
        format_with_options(
            r#"<%= form_with model: user, url: user_path(user), data: { turbo_frame: "profile" } do |form| %><div></div><% end %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%= form_with(\n  model: user,\n  url: user_path(user),\n  data: { turbo_frame: \"profile\" }\n) do |form| %>\n  <div></div>\n<% end %>\n"
    );
}

#[test]
fn wraps_long_link_to_output_command_calls_with_do_blocks() {
    assert_eq!(
        format_with_options(
            r#"<%= link_to "Edit profile", edit_user_path(user), class: "button button--primary", data: { turbo_frame: "_top", controller: "profile-link" } do %><span>Edit</span><% end %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%= link_to(\n  \"Edit profile\",\n  edit_user_path(user),\n  class: \"button button--primary\",\n  data: {\n    turbo_frame: \"_top\",\n    controller: \"profile-link\"\n  }\n) do %>\n  <span>Edit</span>\n<% end %>\n"
    );
}

#[test]
fn wraps_long_parenthesized_erb_output_with_brace_block_suffix() {
    assert_eq!(
        format_with_options(
            r#"<%= content_tag(:span, class: "status-label status-label--active") { t(".active") } %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%= content_tag(\n  :span,\n  class: \"status-label status-label--active\"\n) { t(\".active\") } %>\n"
    );
}

#[test]
fn wraps_long_parenthesized_rails_helper_calls() {
    let options = FormatOptions {
        line_width: 60,
        ..FormatOptions::default()
    };

    assert_eq!(
        format_with_options(
            r#"<%= image_tag("user-placeholder.png", alt: "User profile image", class: "avatar avatar--large") %>"#,
            options
        ),
        "<%= image_tag(\n  \"user-placeholder.png\",\n  alt: \"User profile image\",\n  class: \"avatar avatar--large\"\n) %>\n"
    );

    assert_eq!(
        format_with_options(
            r#"<%= video_tag(["intro.mp4", "intro.webm"], controls: true, autoplay: false, class: "hero-video") %>"#,
            options
        ),
        "<%= video_tag(\n  [\"intro.mp4\", \"intro.webm\"],\n  controls: true,\n  autoplay: false,\n  class: \"hero-video\"\n) %>\n"
    );

    let formatted = format_with_options(
        r#"<%= form_with(model: user, url: user_path(user), data: { turbo_frame: "profile" }) do |form| %><div></div><% end %>"#,
        options,
    );
    let expected = "<%= form_with(\n  model: user,\n  url: user_path(user),\n  data: { turbo_frame: \"profile\" }\n) do |form| %>\n  <div></div>\n<% end %>\n";

    assert_eq!(formatted, expected);
    assert_eq!(format_with_options(&formatted, options), expected);
}

#[test]
fn preserves_long_erb_output_when_expression_is_not_safely_splittable() {
    assert_eq!(
        format_with_options(
            r#"<%= current_user.admin? ? link_to("Admin", admin_path) : link_to("Home", root_path) %>"#,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<%= current_user.admin? ? link_to(\"Admin\", admin_path) : link_to(\"Home\", root_path) %>\n"
    );
}

#[test]
fn wraps_long_modifier_erb_output_while_preserving_suffix() {
    let input =
        r#"<%= hidden_field_tag("referer", h(params[:referer])) unless params[:referer].nil? %>"#;

    assert_eq!(
        format_with_options(
            input,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<%= hidden_field_tag(\n  \"referer\",\n  h(params[:referer])\n) unless params[:referer].nil? %>\n"
    );
}

#[test]
fn preserves_inline_modifier_erb_output_while_preserving_text_boundaries() {
    let input =
        r#"<p>Back to <%= link_to("previous page", back_path) unless back_path.nil? %></p>"#;

    assert_eq!(
        format_with_options(
            input,
            FormatOptions {
                line_width: 48,
                ..FormatOptions::default()
            }
        ),
        "<p>Back to <%= link_to(\"previous page\", back_path) unless back_path.nil? %></p>\n"
    );
}

#[test]
fn preserves_existing_multiline_erb_output_shape() {
    assert_eq!(
        format(
            "<%=\n  link_to(\n    \"Edit profile\",\n    edit_user_path(user),\n    class: \"button\"\n  )\n%>\n"
        ),
        "<%= link_to(\n  \"Edit profile\",\n  edit_user_path(user),\n  class: \"button\"\n) %>\n"
    );
}

#[test]
fn normalizes_multiline_erb_output_opening_marker_inside_html() {
    assert_eq!(
        format_source(
            "<p><%= render(\n  partial: \"partial\",\n  locals: {key: \"value\"}\n) %></p>\n"
        ),
        "<p><%= render(\n  partial: \"partial\",\n  locals: {key: \"value\"}\n) %></p>\n"
    );
}

#[test]
fn wraps_inline_single_foldable_erb_output_child_without_changing_boundaries() {
    let options = FormatOptions {
        line_width: 40,
        ..FormatOptions::default()
    };
    let input = "<p><%= render(partial: \"foo\", locals: { key: \"value\" }) %></p>\n";

    assert_eq!(
        format_source_with_options(input, options),
        "<p><%= render(\n  partial: \"foo\",\n  locals: { key: \"value\" }\n) %></p>\n"
    );
}

#[test]
fn wraps_text_like_element_erb_output_without_inserting_outer_whitespace() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = r#"<h2><%= t ".comment", author: comment.author.display_name, changeset_id: comment.changeset.id.to_s %></h2>
"#;

    assert_eq!(
        format_source_with_options(input, options),
        "<h2><%= t(\n  \".comment\",\n  author: comment.author.display_name,\n  changeset_id: comment.changeset.id.to_s\n) %></h2>\n"
    );
}

#[test]
fn wraps_hashrocket_erb_output_when_expression_is_safely_foldable() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = r#"<h2><%= t ".comment", :author => comment.author.display_name, :changeset_id => comment.changeset.id.to_s %></h2>
"#;

    assert_eq!(
        format_source_with_options(input, options),
        "<h2><%= t(\n  \".comment\",\n  :author => comment.author.display_name,\n  :changeset_id => comment.changeset.id.to_s\n) %></h2>\n"
    );
}

#[test]
fn preserves_hashrocket_multiline_erb_output_indentation_when_not_foldable() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = r#"<h2><%= t ".comment",
          :author => comment.author.display_name,
          :changeset_id => comment.changeset.id.to_s %></h2>
"#;

    assert_eq!(format_source_with_options(input, options), input);
}

#[test]
fn preserves_standalone_hashrocket_multiline_erb_output_indentation_when_not_foldable() {
    let input = r#"<%= inline_svg_tag "auth_providers/example.svg",
                   :class => "rounded-1 text-body-emphasis",
                   :title => t("application.auth_providers.example.alt"),
                   :size => "36" %>
"#;

    assert_eq!(format_source(input), input);
}

#[test]
fn preserves_hashrocket_multiline_erb_output_when_adjacent_to_text_boundaries() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = r#"(<%= t ".also_part_of_html",
       :count => related_ways.size,
       :related_ways => to_sentence(related_ways) %>)
"#;

    assert_eq!(format_source_with_options(input, options), input);
}

#[test]
fn preserves_hashrocket_multiline_erb_block_opener_indentation_when_not_foldable() {
    let input = r#"<%= link_to auth_path(:provider => provider),
            :method => :post,
            :class => ["auth_button btn btn-outline-secondary border p-2", { "px-4 d-flex gap-3 justify-content-center align-items-center" => preferred }],
            :title => t("application.auth_providers.#{provider}.title") do %>
  <span>Sign in</span>
<% end %>
"#;

    assert_eq!(format_source(input), input);
}

#[test]
fn wraps_hashrocket_erb_block_opener_when_expression_is_safely_foldable() {
    let input = r#"<%= link_to auth_path(:provider => provider), :method => :post, :class => "button" do %>
<span>Sign in</span>
<% end %>
"#;
    let expected = r#"<%= link_to(
  auth_path(:provider => provider),
  :method => :post,
  :class => "button"
) do %>
  <span>Sign in</span>
<% end %>
"#;

    assert_eq!(format_source(input), expected);
    assert_eq!(format_source(expected), expected);
}

#[test]
fn wraps_symbol_adjacent_helper_without_changing_inline_boundaries() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = r#"<h4><%= if common_details.redacted? then t("redacted") else t("version") end %>#<%= link_to version, path %></h4>
"#;

    assert_eq!(
        format_source_with_options(input, options),
        "<h4><%= if common_details.redacted? then t(\"redacted\") else t(\"version\") end %>#<%= link_to(\n  version,\n  path\n) %></h4>\n"
    );
}

#[test]
fn preserves_text_adjacent_to_foldable_erb_output_child() {
    let input = "<p>Before <%= render(partial: \"foo\", locals: { key: \"value\" }) %> after</p>\n";

    assert_eq!(
        format_source_with_options(
            input,
            FormatOptions {
                line_width: 40,
                ..FormatOptions::default()
            }
        ),
        "<p>Before <%= render(\n  partial: \"foo\",\n  locals: { key: \"value\" }\n) %> after</p>\n"
    );
}

#[test]
fn wraps_text_adjacent_helper_without_changing_inline_boundaries() {
    let input = "<p>Location <%= link_to(t(\".coordinates_html\", latitude: lat, longitude: lon), root_path(anchor: \"map=18/35/139\")) %></p>\n";

    assert_eq!(
        format_source_with_options(
            input,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<p>Location <%= link_to(\n  t(\".coordinates_html\", latitude: lat, longitude: lon),\n  root_path(anchor: \"map=18/35/139\")\n) %></p>\n"
    );
}

#[test]
fn preserves_forem_text_boundaries_after_folded_erb_output() {
    let heading = r#"<h4><%= pluralize(@automation.errors.count, "error") %> prohibited this automation from being saved:</h4>
"#;
    let button = r#"<button aria-label="Confirm user <%= r.username %> deleted" data-username="<%= r.username %>" data-gdpr-form-action="<%= url_for(action: :destroy, id: r.id) %>" class="c-btn"> <%= crayons_icon_tag("checkmark", aria_hidden: true) %> Deleted</button>
"#;

    assert_eq!(
        format_source(heading),
        "<h4><%= pluralize(\n  @automation.errors.count,\n  \"error\"\n) %> prohibited this automation from being saved:</h4>\n"
    );
    assert_eq!(
        format_source(button),
        "<button aria-label=\"Confirm user <%= r.username %> deleted\" data-username=\"<%= r.username %>\" data-gdpr-form-action=\"<%= url_for(action: :destroy, id: r.id) %>\" class=\"c-btn\"> <%= crayons_icon_tag(\n  \"checkmark\",\n  aria_hidden: true\n) %> Deleted</button>\n"
    );
}

#[test]
fn preserves_forem_closing_tag_boundaries_after_folded_erb_output() {
    let input = r#"<td><%= link_to("@#{user.username}", user.path, class: "crayons-link fw-bold") %></td>
"#;

    assert_eq!(
        format_source(input),
        "<td><%= link_to(\n  \"@#{user.username}\",\n  user.path,\n  class: \"crayons-link fw-bold\"\n) %></td>\n"
    );
}

#[test]
fn preserves_forem_svg_child_boundary_when_wrapping_opening_tag() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
"#;

    assert_eq!(
        format_source(input),
        "<svg\n  xmlns=\"http://www.w3.org/2000/svg\"\n  width=\"14\"\n  height=\"14\"\n  viewBox=\"0 0 24 24\"\n  fill=\"none\"\n  stroke=\"currentColor\"\n  stroke-width=\"2\"\n  stroke-linecap=\"round\"\n  stroke-linejoin=\"round\"\n><line x1=\"18\" y1=\"6\" x2=\"6\" y2=\"18\"></line><line x1=\"6\" y1=\"6\" x2=\"18\" y2=\"18\"></line></svg>\n"
    );
}

#[test]
fn wraps_forem_render_with_chained_collection_scope() {
    let input = r#"<%= render partial: "layouts/sidebar_nav_link", collection: NavigationLink.where(id: @navigation_links[:default_nav_ids]).where.not(name: ["Home", "Reading List"]).ordered, as: :link %>
"#;

    assert_eq!(
        format_source(input),
        "<%= render(\n  partial: \"layouts/sidebar_nav_link\",\n  collection: NavigationLink\n    .where(id: @navigation_links[:default_nav_ids])\n    .where.not(name: [\"Home\", \"Reading List\"])\n    .ordered,\n  as: :link\n) %>\n"
    );
}

#[test]
fn normalizes_forem_multiline_command_helper_continuations() {
    let input = r#"<%= link_to "Delete", admin_subforem_community_bot_scheduled_automation_path(@subforem, @bot, automation), 
    method: :delete, 
    data: { confirm: "Are you sure you want to delete this automation? This action cannot be undone." },
    class: "crayons-btn crayons-btn--danger crayons-btn--s" %>
"#;

    assert_eq!(
        format_source(input),
        "<%= link_to(\n  \"Delete\",\n  admin_subforem_community_bot_scheduled_automation_path(@subforem, @bot, automation),\n  method: :delete,\n  data: { confirm: \"Are you sure you want to delete this automation? This action cannot be undone.\" },\n  class: \"crayons-btn crayons-btn--danger crayons-btn--s\"\n) %>\n"
    );
}

#[test]
fn wraps_forem_link_helpers_with_ternary_keyword_arguments() {
    let input = r#"<li><%= link_to t("views.admin.users.tabs.overview"), admin_user_path(@user.id), class: "crayons-navigation__item #{'crayons-navigation__item crayons-navigation__item--current' if @current_tab == 'overview'}", aria: @current_tab == "overview" ? { current: "page" } : {} %></li>
"#;

    assert_eq!(
        format_source(input),
        "<li><%= link_to(\n  t(\"views.admin.users.tabs.overview\"),\n  admin_user_path(@user.id),\n  class: \"crayons-navigation__item #{'crayons-navigation__item crayons-navigation__item--current' if @current_tab == 'overview'}\",\n  aria: @current_tab == \"overview\" ? { current: \"page\" } : {}\n) %></li>\n"
    );
}

#[test]
fn wraps_forem_select_helpers_with_receiver_block_arguments() {
    let input = r#"<%= f.select :type_of, Survey.type_ofs.keys.map { |type| [type.titleize, type] }, {}, class: "crayons-select__native" %>
"#;

    assert_eq!(
        format_source(input),
        "<%= f.select(\n  :type_of,\n  Survey.type_ofs.keys.map { |type| [type.titleize, type] },\n  {},\n  class: \"crayons-select__native\"\n) %>\n"
    );
}

#[test]
fn wraps_forem_render_helpers_with_modifier_suffixes() {
    let input = r#"<%= render partial: "shared/auth_widget", locals: { tracking_id: "ca_left_sidebar_home_page", source: "left_sidebar" } unless user_signed_in? %>
"#;

    assert_eq!(
        format_source(input),
        "<%= render(\n  partial: \"shared/auth_widget\",\n  locals: {\n    tracking_id: \"ca_left_sidebar_home_page\",\n    source: \"left_sidebar\"\n  }\n) unless user_signed_in? %>\n"
    );
}

#[test]
fn wraps_forem_link_helpers_with_quoted_label_keyword_arguments() {
    let input = r#"<%= link_to "‹", calendar_path(month: prev_month.strftime("%Y-%m"), start: params[:start].presence, end: params[:end].presence, type_of: params[:type_of].presence), class: "calendar__nav-btn", "aria-label": t("views.calendar.nav.previous_month") %>
"#;

    assert_eq!(
        format_source(input),
        "<%= link_to(\n  \"‹\",\n  calendar_path(month: prev_month.strftime(\"%Y-%m\"), start: params[:start].presence, end: params[:end].presence, type_of: params[:type_of].presence),\n  class: \"calendar__nav-btn\",\n  \"aria-label\": t(\"views.calendar.nav.previous_month\")\n) %>\n"
    );
}

#[test]
fn normalizes_existing_multiline_parenthesized_erb_output_arguments() {
    assert_eq!(
        format(
            "<%= react_component(\"ReactComponent\",\n  props: {\n    key1: \"value1\",\n    key2: \"value2\"\n  }\n) %>\n"
        ),
        "<%= react_component(\n  \"ReactComponent\",\n  props: {\n    key1: \"value1\",\n    key2: \"value2\"\n  }\n) %>\n"
    );
}

#[test]
fn formats_if_elsif_else_branches() {
    assert_eq!(
        format(
            "<% if admin? %>\n<p>Admin</p>\n<% elsif user? %>\n<p>User</p>\n<% else %>\n<p>Guest</p>\n<% end %>\n"
        ),
        "<% if admin? %>\n  <p>Admin</p>\n<% elsif user? %>\n  <p>User</p>\n<% else %>\n  <p>Guest</p>\n<% end %>\n"
    );
}

#[test]
fn formats_case_when_branches() {
    assert_eq!(
        format(
            "<% case role %>\n<% when \"admin\" %>\n<p>Admin</p>\n<% when \"user\" %>\n<p>User</p>\n<% end %>\n"
        ),
        "<% case role %>\n<% when \"admin\" %>\n  <p>Admin</p>\n<% when \"user\" %>\n  <p>User</p>\n<% end %>\n"
    );
}

#[test]
fn formats_output_erb_do_blocks() {
    assert_eq!(
        format(
            "<%= form_with model: user do |form| %>\n<div>\n<%= form.text_field :name %>\n</div>\n<% end %>\n"
        ),
        "<%= form_with model: user do |form| %>\n  <div>\n    <%= form.text_field :name %>\n  </div>\n<% end %>\n"
    );
}

#[test]
fn formats_begin_rescue_ensure_branches() {
    assert_eq!(
        format(
            "<% begin %>\n<p>Saving</p>\n<% rescue => error %>\n<p>Failed</p>\n<% ensure %>\n<p>Done</p>\n<% end %>\n"
        ),
        "<% begin %>\n  <p>Saving</p>\n<% rescue => error %>\n  <p>Failed</p>\n<% ensure %>\n  <p>Done</p>\n<% end %>\n"
    );
}

#[test]
fn snapshots_default_html_indentation() {
    insta::assert_snapshot!(
        "default_html_indentation",
        format(
            "<div>\n<h1><%= page_title %></h1>\n<% if user %>\n<p>Hello, <%= user.name %></p>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n</div>\n"
        )
    );
}

#[test]
fn snapshots_without_html_indentation() {
    insta::assert_snapshot!(
        "without_html_indentation",
        format_without_html_indent(
            "<div>\n<h1><%= page_title %></h1>\n<% if user %>\n<p>Hello, <%= user.name %></p>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n</div>\n"
        )
    );
}

#[test]
fn snapshots_branch_formatting() {
    insta::assert_snapshot!(
        "branch_formatting",
        format(
            "<% if admin? %>\n<p>Admin</p>\n<% elsif user? %>\n<p>User</p>\n<% else %>\n<p>Guest</p>\n<% end %>\n<% case role %>\n<% when \"admin\" %>\n<p>Admin tools</p>\n<% when \"user\" %>\n<p>User dashboard</p>\n<% end %>\n"
        )
    );
}

#[test]
fn snapshots_stability_fixture() {
    insta::assert_snapshot!("stability_fixture", format(stability_fixture()));
}

#[test]
fn snapshots_stability_fixture_without_html_indentation() {
    insta::assert_snapshot!(
        "stability_fixture_without_html_indentation",
        format_without_html_indent(stability_fixture())
    );
}

#[test]
fn snapshots_formatter_audit_fixture() {
    insta::assert_snapshot!("formatter_audit_fixture", format(formatter_audit_fixture()));
}

#[test]
fn snapshots_formatter_edge_cases_fixture() {
    insta::assert_snapshot!(
        "formatter_edge_cases_fixture",
        format_source(formatter_edge_cases_fixture())
    );
}

#[test]
fn snapshots_real_template_audit_fixture() {
    insta::assert_snapshot!(
        "real_template_audit_fixture",
        format(real_template_audit_fixture())
    );
}

#[test]
fn formatted_sample_fixture_is_idempotent() {
    assert_format_is_idempotent(sample_fixture());
}

#[test]
fn formatted_stability_fixture_is_idempotent() {
    assert_format_is_idempotent(stability_fixture());
}

#[test]
fn formatted_formatter_audit_fixture_is_idempotent() {
    assert_format_is_idempotent(formatter_audit_fixture());
}

#[test]
fn formatted_formatter_edge_cases_fixture_is_idempotent() {
    assert_format_source_is_idempotent(formatter_edge_cases_fixture());
}

#[test]
fn formatted_real_template_audit_fixture_is_idempotent() {
    assert_format_is_idempotent(real_template_audit_fixture());
}

fn assert_format_is_idempotent(input: &str) {
    let once = format(input);
    let twice = format(&once);

    assert_eq!(twice, once);
}

fn assert_format_source_is_idempotent(input: &str) {
    let once = format_source(input);
    let twice = format_source(&once);

    assert_eq!(twice, once);
}

fn sample_fixture() -> &'static str {
    include_str!("../../samples/sample.html.erb")
}

fn stability_fixture() -> &'static str {
    "<!DOCTYPE html>\n<div class=\"page <%= page_class %>\">\n<!-- profile card -->\n<img src=\"<%= avatar_url %>\" alt=\"<%= user.name %>\">\n<input type=\"checkbox\" checked=\"<%= checked %>\">\n<% if user %>\n<section>\n<a href=\"/users/<%= user.id %>\"><%= user.name %></a>\n<br>\n<% unless notifications.empty? %>\n<ul>\n<% notifications.each do |notification| %>\n<li><%= notification.title %></li>\n<% end %>\n</ul>\n<% end %>\n</section>\n<% else %>\n<p>Please sign in</p>\n<% end %>\n</div>\n"
}

fn formatter_audit_fixture() -> &'static str {
    include_str!("../../samples/formatter-audit.html.erb")
}

fn formatter_edge_cases_fixture() -> &'static str {
    include_str!("../../samples/formatter-edge-cases.html.erb")
}

fn real_template_audit_fixture() -> &'static str {
    include_str!("../../samples/real-template-audit.html.erb")
}

fn formatter_ignore_fixture() -> &'static str {
    include_str!("../../samples/formatter-ignore-next.html.erb")
}
