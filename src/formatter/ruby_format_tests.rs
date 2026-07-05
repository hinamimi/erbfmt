use super::*;
use crate::syntax::{
    erb::{tokenize, tokenize_with_spans},
    template::{parse, parse_spanned},
};

fn format(input: &str) -> String {
    let tokens = tokenize(input).unwrap();
    let document = parse(&tokens).unwrap();

    format_document(&document)
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
fn safety_preserves_visible_whitespace_around_inline_helpers() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = r#"<p>Saved by <%= link_to user.name, user_path(user), class: "profile-link profile-link--long" %> in <%= link_to project.name, project_path(project), class: "project-link project-link--long" %>.</p>
"#;
    let expected = "<p>Saved by <%= link_to user.name, user_path(user), class: \"profile-link profile-link--long\" %> in <%= link_to project.name, project_path(project), class: \"project-link project-link--long\" %>.</p>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
}

#[test]
fn safety_preserves_inline_boundaries_around_modifier_suffix_helpers() {
    let options = FormatOptions {
        line_width: 48,
        ..FormatOptions::default()
    };
    let input = r#"<p>Before <%= hidden_field_tag("referer", h(params[:really_long_referer])) unless params[:referer].nil? %> after</p>
"#;
    let expected = "<p>Before <%= hidden_field_tag(\n  \"referer\",\n  h(params[:really_long_referer])\n) unless params[:referer].nil? %> after</p>\n";

    assert_eq!(format_source_with_options(input, options), expected);
    assert_eq!(format_source_with_options(expected, options), expected);
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
fn wraps_nested_helper_calls_inside_keyword_arguments() {
    let input = r#"<%= t("views.admin.users.actions.adjust_credit.selector_html",
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
"#;

    assert_eq!(
        format_source(input),
        "<%= t(\n  \"views.admin.users.actions.adjust_credit.selector_html\",\n  action: f.select(\n    :credit_action,\n    options_for_select({ t(\"views.admin.users.actions.adjust_credit.add\") => \"Add\" }),\n    {},\n    class: \"js-credit-action crayons-select w-auto\"\n  ),\n  amount: f.text_field(\n    :credit_amount,\n    type: \"number\",\n    required: true,\n    class: \"js-credit-amount crayons-textfield w-auto\",\n    value: 1,\n    size: 5,\n    min: 1,\n    max: 9999\n  ),\n  credits: f.label(\n    :credit_amount,\n    t(\"views.admin.users.actions.adjust_credit.unit\"),\n    class: \"color-secondary flex-1\",\n    \"aria-label\": t(\"views.admin.users.actions.adjust_credit.aria_credits\")\n  )\n) %>\n"
    );
}

#[test]
fn wraps_nested_helper_calls_inside_keyword_hash_entries() {
    let input = r#"<%= render partial: "card", locals: { action: link_to("Edit profile", edit_user_profile_settings_path(user), class: "button button--primary"), label: t(".label") } %>
"#;

    assert_eq!(
        format_source(input),
        "<%= render(\n  partial: \"card\",\n  locals: {\n    action: link_to(\n      \"Edit profile\",\n      edit_user_profile_settings_path(user),\n      class: \"button button--primary\"\n    ),\n    label: t(\".label\")\n  }\n) %>\n"
    );
}

#[test]
fn wraps_long_keyword_array_arguments() {
    let input = r#"<%= tag.div "Content", class: ["profile-card", "profile-card--featured", current_user.admin? ? "profile-card--admin" : nil, extra_class] %>
"#;

    assert_eq!(
        format_source(input),
        "<%= tag.div(\n  \"Content\",\n  class: [\n    \"profile-card\",\n    \"profile-card--featured\",\n    current_user.admin? ? \"profile-card--admin\" : nil,\n    extra_class\n  ]\n) %>\n"
    );
}

#[test]
fn safety_keeps_quoted_label_and_nested_helper_wrapping_idempotent() {
    let input = r#"<%= link_to "Save", admin_path(filters: { state: params[:state] }, action: f.select(:credit_action, options_for_select({ t("views.admin.users.actions.adjust_credit.add") => "Add" }), {}, class: "js-credit-action crayons-select w-auto")), class: "crayons-btn crayons-btn--primary", "aria-label": t("views.admin.users.actions.adjust_credit.aria_save") %>
"#;
    let expected = "<%= link_to(\n  \"Save\",\n  admin_path(filters: { state: params[:state] }, action: f.select(:credit_action, options_for_select({ t(\"views.admin.users.actions.adjust_credit.add\") => \"Add\" }), {}, class: \"js-credit-action crayons-select w-auto\")),\n  class: \"crayons-btn crayons-btn--primary\",\n  \"aria-label\": t(\"views.admin.users.actions.adjust_credit.aria_save\")\n) %>\n";

    assert_eq!(format_source(input), expected);
    assert_eq!(format_source(expected), expected);
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
