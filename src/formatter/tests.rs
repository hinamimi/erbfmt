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
        "<!-- erbfmt-ignore format: separated -->\n\n<article\n  class=\"card\"\n  data-controller=\"profile\"\n  aria-label=\"Profile card\"\n>\n</article>\n"
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
        format(
            "<a\nhref=\"/users/<%= user.id %>\"\naria-label=\"<%= user.name %>\"\n>Profile</a>\n"
        ),
        "<a\n  href=\"/users/<%= user.id %>\"\n  aria-label=\"<%= user.name %>\"\n>\n  Profile\n</a>\n"
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
        "<%=\n  link_to(\n    \"Edit profile\",\n    edit_user_path(user),\n    class: \"button button--primary\",\n    data: { turbo_frame: \"_top\" }\n  )\n%>\n"
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
fn preserves_long_erb_block_opening_tags_when_they_are_control_flow() {
    assert_eq!(
        format_with_options(
            "<% if current_user.admin? && feature_enabled?(:new_dashboard) && account.active? %>\n<p>Hello</p>\n<% end %>\n",
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%\n  if current_user.admin? && feature_enabled?(:new_dashboard) && account.active?\n%>\n  <p>Hello</p>\n<% end %>\n"
    );
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
fn wraps_long_erb_output_command_calls_with_do_blocks() {
    assert_eq!(
        format_with_options(
            r#"<%= form_with model: user, url: user_path(user), data: { turbo_frame: "profile" } do |form| %><div></div><% end %>"#,
            FormatOptions {
                line_width: 60,
                ..FormatOptions::default()
            }
        ),
        "<%=\n  form_with(\n    model: user,\n    url: user_path(user),\n    data: { turbo_frame: \"profile\" }\n  ) do |form|\n%>\n  <div></div>\n<% end %>\n"
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
        "<%=\n  image_tag(\n    \"user-placeholder.png\",\n    alt: \"User profile image\",\n    class: \"avatar avatar--large\"\n  )\n%>\n"
    );

    assert_eq!(
        format_with_options(
            r#"<%= video_tag(["intro.mp4", "intro.webm"], controls: true, autoplay: false, class: "hero-video") %>"#,
            options
        ),
        "<%=\n  video_tag(\n    [\"intro.mp4\", \"intro.webm\"],\n    controls: true,\n    autoplay: false,\n    class: \"hero-video\"\n  )\n%>\n"
    );

    let formatted = format_with_options(
        r#"<%= form_with(model: user, url: user_path(user), data: { turbo_frame: "profile" }) do |form| %><div></div><% end %>"#,
        options,
    );
    let expected = "<%=\n  form_with(\n    model: user,\n    url: user_path(user),\n    data: { turbo_frame: \"profile\" }\n  ) do |form|\n%>\n  <div></div>\n<% end %>\n";

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
        "<%=\n  current_user.admin? ? link_to(\"Admin\", admin_path) : link_to(\"Home\", root_path)\n%>\n"
    );
}

#[test]
fn preserves_existing_multiline_erb_output_shape() {
    assert_eq!(
        format(
            "<%=\n  link_to(\n    \"Edit profile\",\n    edit_user_path(user),\n    class: \"button\"\n  )\n%>\n"
        ),
        "<%=\n  link_to(\n    \"Edit profile\",\n    edit_user_path(user),\n    class: \"button\"\n  )\n%>\n"
    );
}

#[test]
fn normalizes_multiline_erb_output_opening_marker_inside_html() {
    assert_eq!(
        format("<p><%= render(\n  partial: \"partial\",\n  locals: {key: \"value\"}\n) %></p>\n"),
        "<p>\n  <%=\n    render(\n      partial: \"partial\",\n      locals: {key: \"value\"}\n    )\n  %>\n</p>\n"
    );
}

#[test]
fn normalizes_existing_multiline_parenthesized_erb_output_arguments() {
    assert_eq!(
        format(
            "<%= react_component(\"ReactComponent\",\n  props: {\n    key1: \"value1\",\n    key2: \"value2\"\n  }\n) %>\n"
        ),
        "<%=\n  react_component(\n    \"ReactComponent\",\n    props: {\n      key1: \"value1\",\n      key2: \"value2\"\n    }\n  )\n%>\n"
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
        format(formatter_edge_cases_fixture())
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
    assert_format_is_idempotent(formatter_edge_cases_fixture());
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
