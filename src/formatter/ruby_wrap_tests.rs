use super::ruby_wrap::fold_command_call;

#[test]
fn folds_command_call_arguments() {
    assert_eq!(
        fold_command_call(r#"link_to "Edit profile", edit_user_path(user), class: "button""#),
        Some(vec![
            "link_to(".to_string(),
            r#"  "Edit profile","#.to_string(),
            "  edit_user_path(user),".to_string(),
            r#"  class: "button""#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_command_call_with_do_block() {
    assert_eq!(
        fold_command_call(r#"form_with model: user, url: user_path(user) do |form|"#),
        Some(vec![
            "form_with(".to_string(),
            "  model: user,".to_string(),
            "  url: user_path(user)".to_string(),
            ") do |form|".to_string(),
        ])
    );
}

#[test]
fn folds_command_call_with_modifier_suffix() {
    assert_eq!(
        fold_command_call(
            r#"render partial: "shared/auth_widget", locals: { tracking_id: "ca_left_sidebar_home_page", source: "left_sidebar" } unless user_signed_in?"#
        ),
        Some(vec![
            "render(".to_string(),
            r#"  partial: "shared/auth_widget","#.to_string(),
            "  locals: {".to_string(),
            r#"    tracking_id: "ca_left_sidebar_home_page","#.to_string(),
            r#"    source: "left_sidebar""#.to_string(),
            "  }".to_string(),
            ") unless user_signed_in?".to_string(),
        ])
    );
}

#[test]
fn folds_command_call_with_legacy_hash_pair_arguments() {
    assert_eq!(
        fold_command_call(
            r#"link_to "Edit", edit_user_path(user), :method => :post, :class => "button""#
        ),
        Some(vec![
            "link_to(".to_string(),
            r#"  "Edit","#.to_string(),
            "  edit_user_path(user),".to_string(),
            "  :method => :post,".to_string(),
            r#"  :class => "button""#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_existing_multiline_command_call_continuations() {
    assert_eq!(
        fold_command_call(
            r#"link_to "Delete", admin_path(record),
                  method: :delete,
                  data: { confirm: "Are you sure?" },
                  class: "button button--danger""#
        ),
        Some(vec![
            "link_to(".to_string(),
            r#"  "Delete","#.to_string(),
            "  admin_path(record),".to_string(),
            "  method: :delete,".to_string(),
            r#"  data: { confirm: "Are you sure?" },"#.to_string(),
            r#"  class: "button button--danger""#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_ternary_expressions_inside_call_arguments() {
    assert_eq!(
        fold_command_call(
            r#"link_to t("tabs.overview"), admin_user_path(user), class: "tab", aria: current_tab == "overview" ? { current: "page" } : {}"#
        ),
        Some(vec![
            "link_to(".to_string(),
            r#"  t("tabs.overview"),"#.to_string(),
            "  admin_user_path(user),".to_string(),
            r#"  class: "tab","#.to_string(),
            r#"  aria: current_tab == "overview" ? { current: "page" } : {}"#.to_string(),
            ")".to_string(),
        ])
    );

    assert_eq!(
        fold_command_call(r#"current_tab == "overview" ? "on" : "off""#),
        None
    );
}

#[test]
fn folds_command_call_with_legacy_hash_pair_do_block_arguments() {
    assert_eq!(
        fold_command_call(
            r#"link_to auth_path(:provider => provider), :method => :post, :class => "button" do"#
        ),
        Some(vec![
            "link_to(".to_string(),
            "  auth_path(:provider => provider),".to_string(),
            "  :method => :post,".to_string(),
            r#"  :class => "button""#.to_string(),
            ") do".to_string(),
        ])
    );
}

#[test]
fn folds_parenthesized_call_arguments() {
    assert_eq!(
        fold_command_call(
            r#"video_tag(["intro.mp4", "intro.webm"], controls: true, class: "hero-video")"#
        ),
        Some(vec![
            "video_tag(".to_string(),
            r#"  ["intro.mp4", "intro.webm"],"#.to_string(),
            "  controls: true,".to_string(),
            r#"  class: "hero-video""#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_parenthesized_call_with_do_block() {
    assert_eq!(
        fold_command_call("form_with(model: user, url: user_path(user)) do |form|"),
        Some(vec![
            "form_with(".to_string(),
            "  model: user,".to_string(),
            "  url: user_path(user)".to_string(),
            ") do |form|".to_string(),
        ])
    );
}

#[test]
fn folds_parenthesized_call_with_brace_block() {
    assert_eq!(
        fold_command_call(r#"content_tag(:span, class: "label") { t(".label") }"#),
        Some(vec![
            "content_tag(".to_string(),
            "  :span,".to_string(),
            r#"  class: "label""#.to_string(),
            r#") { t(".label") }"#.to_string(),
        ])
    );
}

#[test]
fn folds_parenthesized_call_with_brace_block_params() {
    assert_eq!(
        fold_command_call(r#"tag.ul(items, class: "list") { |item| tag.li(item.name) }"#),
        Some(vec![
            "tag.ul(".to_string(),
            "  items,".to_string(),
            r#"  class: "list""#.to_string(),
            r#") { |item| tag.li(item.name) }"#.to_string(),
        ])
    );
}

#[test]
fn folds_keyword_hash_arguments() {
    assert_eq!(
        fold_command_call(
            r#"render partial: "profile", locals: { current_user: current_user, account: account, selected_status: selected_status }"#
        ),
        Some(vec![
            "render(".to_string(),
            r#"  partial: "profile","#.to_string(),
            "  locals: {".to_string(),
            "    current_user: current_user,".to_string(),
            "    account: account,".to_string(),
            "    selected_status: selected_status".to_string(),
            "  }".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_nested_helper_calls_inside_keyword_hash_entries() {
    assert_eq!(
        fold_command_call(
            r#"render partial: "card", locals: { action: link_to("Edit profile", edit_user_profile_settings_path(user), class: "button button--primary"), label: t(".label") }"#
        ),
        Some(vec![
            "render(".to_string(),
            r#"  partial: "card","#.to_string(),
            "  locals: {".to_string(),
            "    action: link_to(".to_string(),
            r#"      "Edit profile","#.to_string(),
            "      edit_user_profile_settings_path(user),".to_string(),
            r#"      class: "button button--primary""#.to_string(),
            "    ),".to_string(),
            r#"    label: t(".label")"#.to_string(),
            "  }".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_nested_helper_calls_inside_hashrocket_entries() {
    assert_eq!(
        fold_command_call(
            r#"render locals: { :action => link_to("Edit profile", edit_user_profile_settings_path(user), class: "button button--primary"), :label => t(".label") }"#
        ),
        Some(vec![
            "render(".to_string(),
            "  locals: {".to_string(),
            "    :action => link_to(".to_string(),
            r#"      "Edit profile","#.to_string(),
            "      edit_user_profile_settings_path(user),".to_string(),
            r#"      class: "button button--primary""#.to_string(),
            "    ),".to_string(),
            r#"    :label => t(".label")"#.to_string(),
            "  }".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_safe_navigation_chained_method_arguments() {
    assert_eq!(
        fold_command_call(
            r#"render partial: "team", collection: current_user&.organization&.teams&.visible&.for_sidebar_navigation&.ordered_by_name, as: :team"#
        ),
        Some(vec![
            "render(".to_string(),
            r#"  partial: "team","#.to_string(),
            "  collection: current_user".to_string(),
            "    &.organization".to_string(),
            "    &.teams".to_string(),
            "    &.visible".to_string(),
            "    &.for_sidebar_navigation".to_string(),
            "    &.ordered_by_name,".to_string(),
            "  as: :team".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_long_keyword_array_arguments() {
    assert_eq!(
        fold_command_call(
            r#"tag.div "Content", class: ["profile-card", "profile-card--featured", current_user.admin? ? "profile-card--admin" : nil, extra_class]"#
        ),
        Some(vec![
            "tag.div(".to_string(),
            r#"  "Content","#.to_string(),
            "  class: [".to_string(),
            r#"    "profile-card","#.to_string(),
            r#"    "profile-card--featured","#.to_string(),
            r#"    current_user.admin? ? "profile-card--admin" : nil,"#.to_string(),
            "    extra_class".to_string(),
            "  ]".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn keeps_short_keyword_array_arguments_inline() {
    assert_eq!(
        fold_command_call(r#"render partial: "profile", formats: [:html, :turbo_stream]"#),
        Some(vec![
            "render(".to_string(),
            r#"  partial: "profile","#.to_string(),
            "  formats: [:html, :turbo_stream]".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_parenthesized_call_with_trailing_commas() {
    assert_eq!(
        fold_command_call(
            r#"render(partial: "profile", locals: { current_user: current_user, account: account, }, formats: [:html, :turbo_stream,],)"#
        ),
        Some(vec![
            "render(".to_string(),
            r#"  partial: "profile","#.to_string(),
            "  locals: {".to_string(),
            "    current_user: current_user,".to_string(),
            "    account: account".to_string(),
            "  },".to_string(),
            "  formats: [:html, :turbo_stream,]".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_single_keyword_hash_argument() {
    assert_eq!(
        fold_command_call(
            r#"render locals: { current_user: current_user, account: account, selected_status: selected_status }"#
        ),
        Some(vec![
            "render(".to_string(),
            "  locals: {".to_string(),
            "    current_user: current_user,".to_string(),
            "    account: account,".to_string(),
            "    selected_status: selected_status".to_string(),
            "  }".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_existing_multiline_parenthesized_call_arguments() {
    assert_eq!(
        fold_command_call(
            "react_component(\"ReactComponent\",\n  props: {\n    key1: \"value1\",\n    key2: \"value2\"\n  }\n)"
        ),
        Some(vec![
            "react_component(".to_string(),
            r#"  "ReactComponent","#.to_string(),
            "  props: {".to_string(),
            r#"    key1: "value1","#.to_string(),
            r#"    key2: "value2""#.to_string(),
            "  }".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn ignores_commas_inside_strings_and_nested_values() {
    assert_eq!(
        fold_command_call(r#"link_to "Edit, profile", user_path(user, anchor: "top")"#),
        Some(vec![
            "link_to(".to_string(),
            r#"  "Edit, profile","#.to_string(),
            r#"  user_path(user, anchor: "top")"#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_arguments_with_hash_characters_inside_strings() {
    assert_eq!(
        fold_command_call(r##"link_to "Issue #1", issue_path(issue), class: "link""##),
        Some(vec![
            "link_to(".to_string(),
            r##"  "Issue #1","##.to_string(),
            "  issue_path(issue),".to_string(),
            r#"  class: "link""#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_binary_division_arguments_when_ast_can_parse_them() {
    assert_eq!(
        fold_command_call(r#"number_to_percentage completed / total, precision: 2"#),
        Some(vec![
            "number_to_percentage(".to_string(),
            "  completed / total,".to_string(),
            "  precision: 2".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_percent_regex_literals() {
    assert_eq!(
        fold_command_call(r#"highlight body, pattern: %r{/users/\d+}"#),
        Some(vec![
            "highlight(".to_string(),
            "  body,".to_string(),
            r#"  pattern: %r{/users/\d+}"#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_splat_keyword_splat_and_block_arguments() {
    assert_eq!(
        fold_command_call("render(*partials, **options, &block)"),
        Some(vec![
            "render(".to_string(),
            "  *partials,".to_string(),
            "  **options,".to_string(),
            "  &block".to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_constant_paths_and_keyword_names_that_are_ruby_keywords() {
    assert_eq!(
        fold_command_call(r#"component Admin::UserCard, class: "card""#),
        Some(vec![
            "component(".to_string(),
            "  Admin::UserCard,".to_string(),
            r#"  class: "card""#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn folds_quoted_symbol_hash_pair_arguments() {
    assert_eq!(
        fold_command_call(r#"link_to "Save", save_path, :"data-action" => "click->save#submit""#),
        Some(vec![
            "link_to(".to_string(),
            r#"  "Save","#.to_string(),
            "  save_path,".to_string(),
            r#"  :"data-action" => "click->save#submit""#.to_string(),
            ")".to_string(),
        ])
    );
}

#[test]
fn does_not_fold_control_flow() {
    assert_eq!(
        fold_command_call("if current_user.admin? && account.active?"),
        None
    );
}

#[test]
fn does_not_fold_assignment_or_top_level_ternary() {
    assert_eq!(
        fold_command_call(r#"message = t(".saved"), class: "notice""#),
        None
    );
    assert_eq!(
        fold_command_call(r#"current_user.admin? ? admin_path : root_path"#),
        None
    );
}

#[test]
fn does_not_fold_unsupported_block_parameter_shapes() {
    assert_eq!(
        fold_command_call(r#"tag.ul(items, class: "list") { |(key, value)| tag.li(key) }"#),
        None
    );
}

#[test]
fn does_not_fold_single_argument_calls() {
    assert_eq!(
        fold_command_call(r#"cache ["profile-card", user.cache_key_with_version]"#),
        None
    );
}

#[test]
fn does_not_fold_regex_literal_arguments() {
    assert_eq!(
        fold_command_call(r#"helper /a,b/, class: "button button--primary""#),
        None
    );
}

#[test]
fn does_not_fold_heredoc_expressions() {
    assert_eq!(fold_command_call("message = <<TEXT\nhello\nTEXT"), None);
}

#[test]
fn does_not_fold_multistatement_command_calls() {
    assert_eq!(
        fold_command_call(
            "content_for :header, render(\"header\")\n  content_for :footer, render(\"footer\")"
        ),
        None
    );
}

#[test]
fn does_not_fold_single_entry_keyword_hash_arguments() {
    assert_eq!(fold_command_call(r#"render locals: { user: user }"#), None);
}

#[test]
fn does_not_fold_unbalanced_code() {
    assert_eq!(
        fold_command_call(r#"link_to "Edit", edit_user_path(user"#),
        None
    );
}

#[test]
fn does_not_fold_parenthesized_call_with_trailing_expression() {
    assert_eq!(
        fold_command_call(r#"image_tag("profile.png", alt: "Profile") || fallback"#),
        None
    );
}
