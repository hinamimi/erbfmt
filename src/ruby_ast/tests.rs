use super::ast::{
    Argument, AssignmentOperator, BinaryOperator, CallStyle, Expression, HashKey, HashPair,
    MemberOperator, ModifierKeyword, RangeOperator, UnaryOperator,
};
use super::parser::parse;

#[test]
fn parses_parenthesized_helper_call() {
    let program = parse(
        r#"render(partial: "profile", locals: { current_user: current_user, account: account })"#,
    )
    .unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.callee, "render");
    assert_eq!(call.style, CallStyle::Parenthesized);
    assert_eq!(call.arguments.len(), 2);

    assert!(matches!(
        &call.arguments[0],
        Argument::Keyword { name, value: Expression::String(value) }
        if name == "partial" && value == "\"profile\""
    ));
}

#[test]
fn parses_command_style_call_with_do_block() {
    let program = parse(r#"form_with model: user, url: user_path(user) do |form|"#).unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.callee, "form_with");
    assert_eq!(call.style, CallStyle::Command);
    assert_eq!(call.arguments.len(), 2);
    assert_eq!(call.block.unwrap().params, vec!["form"]);
}

#[test]
fn parses_parenthesized_helper_call_with_brace_block() {
    let program = parse(r#"content_tag(:span, class: "label") { t(".label") }"#).unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.callee, "content_tag");
    assert_eq!(call.style, CallStyle::Parenthesized);
    assert_eq!(call.arguments.len(), 2);
    assert!(call.block.is_some());
}

#[test]
fn parses_parenthesized_helper_call_with_brace_block_params() {
    let program = parse(r#"tag.ul(items) { |item| tag.li(item.name) }"#).unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.callee, "tag.ul");
    assert_eq!(call.style, CallStyle::Parenthesized);
    assert_eq!(call.block.unwrap().params, vec!["item"]);
}

#[test]
fn parses_command_style_call_with_legacy_hash_pair_arguments() {
    let program =
        parse(r#"link_to "Edit", edit_user_path(user), :method => :post, :class => "button""#)
            .unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.callee, "link_to");
    assert_eq!(call.style, CallStyle::Command);
    assert_eq!(call.arguments.len(), 4);
    assert!(matches!(
        &call.arguments[2],
        Argument::HashPair {
            key: Expression::Symbol(key),
            value: Expression::Symbol(value),
        } if key == ":method" && value == ":post"
    ));
    assert!(matches!(
        &call.arguments[3],
        Argument::HashPair {
            key: Expression::Symbol(key),
            value: Expression::String(value),
        } if key == ":class" && value == "\"button\""
    ));
}

#[test]
fn parses_trailing_commas_in_helper_arguments_and_literals() {
    let program = parse(
        r#"render(partial: "profile", locals: { current_user: current_user, account: account, }, formats: [:html, :turbo_stream,],)"#,
    )
    .unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.callee, "render");
    assert_eq!(call.arguments.len(), 3);

    let Argument::Keyword {
        value: Expression::Hash(pairs),
        ..
    } = &call.arguments[1]
    else {
        panic!("expected keyword hash argument");
    };

    assert_eq!(pairs.len(), 2);

    let Argument::Keyword {
        value: Expression::Array(values),
        ..
    } = &call.arguments[2]
    else {
        panic!("expected keyword array argument");
    };

    assert_eq!(values.len(), 2);
}

#[test]
fn parses_nested_array_and_hash_arguments() {
    let program = parse(
        r#"video_tag(["intro.mp4", "intro.webm"], data: { controller: :player, enabled: true })"#,
    )
    .unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert!(matches!(
        &call.arguments[0],
        Argument::Positional(Expression::Array(values)) if values.len() == 2
    ));

    let Argument::Keyword {
        value: Expression::Hash(pairs),
        ..
    } = &call.arguments[1]
    else {
        panic!("expected keyword hash argument");
    };

    assert_eq!(pairs.len(), 2);
    assert!(matches!(
        &pairs[0],
        HashPair::KeyValue { key: HashKey::Label(key), .. } if key == "controller"
    ));
}

#[test]
fn parses_semicolon_separated_statements_without_formatting_them() {
    let program = parse(r#"foo; bar"#).unwrap();

    assert_eq!(program.statements.len(), 2);
    assert!(!program.is_format_candidate());
    assert!(matches!(
        program.statements[0],
        Expression::Identifier(ref value) if value == "foo"
    ));
    assert!(matches!(
        program.statements[1],
        Expression::Identifier(ref value) if value == "bar"
    ));
}

#[test]
fn parses_newline_separated_statements_without_formatting_them() {
    let program = parse("user = current_user\npolicy = UserPolicy.new(user)").unwrap();

    assert_eq!(program.statements.len(), 2);
    assert!(!program.is_format_candidate());
    assert!(matches!(program.statements[0], Expression::Assignment(_)));
    assert!(matches!(program.statements[1], Expression::Assignment(_)));
}

#[test]
fn parses_simple_assignment_as_non_format_candidate() {
    let program = parse("user_name = current_user.name").unwrap();

    let Expression::Assignment(assignment) = single_statement(&program) else {
        panic!("expected assignment expression");
    };

    assert_eq!(assignment.target, "user_name");
    assert_eq!(assignment.operator, AssignmentOperator::Assign);
    assert!(!Expression::Assignment(assignment).is_format_candidate());
}

#[test]
fn parses_compound_assignment() {
    let program = parse("count += 1").unwrap();

    let Expression::Assignment(assignment) = single_statement(&program) else {
        panic!("expected assignment expression");
    };

    assert_eq!(assignment.target, "count");
    assert_eq!(assignment.operator, AssignmentOperator::Add);
    assert!(matches!(*assignment.value, Expression::Literal(ref value) if value == "1"));
}

#[test]
fn parses_multiple_assignment_target_without_formatting_it() {
    let program = parse("a, b = values").unwrap();

    let Expression::Assignment(assignment) = single_statement(&program) else {
        panic!("expected assignment expression");
    };

    assert_eq!(assignment.target, "a, b");
    assert!(!Expression::Assignment(assignment).is_format_candidate());
}

#[test]
fn parses_binary_operator_expression() {
    let program = parse("current_user.admin? && account.active?").unwrap();

    let Expression::Binary(binary) = single_statement(&program) else {
        panic!("expected binary expression");
    };

    assert_eq!(binary.operator, BinaryOperator::And);
    assert!(matches!(
        *binary.left,
        Expression::MemberAccess(ref access)
            if access.operator == MemberOperator::Dot
                && access.member == "admin?"
                && matches!(*access.receiver, Expression::Identifier(ref value) if value == "current_user")
    ));
    assert!(matches!(
        *binary.right,
        Expression::MemberAccess(ref access)
            if access.operator == MemberOperator::Dot
                && access.member == "active?"
                && matches!(*access.receiver, Expression::Identifier(ref value) if value == "account")
    ));
}

#[test]
fn parses_binary_operator_precedence() {
    let program = parse("price + tax * quantity").unwrap();

    let Expression::Binary(binary) = single_statement(&program) else {
        panic!("expected binary expression");
    };

    assert_eq!(binary.operator, BinaryOperator::Add);
    assert!(matches!(*binary.left, Expression::Identifier(ref value) if value == "price"));

    let Expression::Binary(right) = *binary.right else {
        panic!("expected nested multiplication");
    };

    assert_eq!(right.operator, BinaryOperator::Multiply);
}

#[test]
fn parses_grouped_operator_expression() {
    let program = parse("(admin? || owner?) && account.active?").unwrap();

    let Expression::Binary(binary) = single_statement(&program) else {
        panic!("expected binary expression");
    };

    assert_eq!(binary.operator, BinaryOperator::And);
    assert!(matches!(*binary.left, Expression::Grouped(_)));
}

#[test]
fn parses_unary_operator_expression() {
    let program = parse("!user.disabled?").unwrap();

    let Expression::Unary(unary) = single_statement(&program) else {
        panic!("expected unary expression");
    };

    assert_eq!(unary.operator, UnaryOperator::Not);
    assert!(matches!(
        *unary.expression,
        Expression::MemberAccess(ref access)
            if access.operator == MemberOperator::Dot
                && access.member == "disabled?"
                && matches!(*access.receiver, Expression::Identifier(ref value) if value == "user")
    ));
}

#[test]
fn parses_assignment_value_as_operator_expression() {
    let program = parse("visible = current_user.admin? && account.active?").unwrap();

    let Expression::Assignment(assignment) = single_statement(&program) else {
        panic!("expected assignment expression");
    };

    assert_eq!(assignment.target, "visible");
    assert!(matches!(*assignment.value, Expression::Binary(_)));
}

#[test]
fn parses_modifier_if_expression_without_formatting_it() {
    let program = parse(r#""active" if user.active?"#).unwrap();

    let Expression::Modifier(modifier) = single_statement(&program) else {
        panic!("expected modifier expression");
    };

    assert_eq!(modifier.keyword, ModifierKeyword::If);
    assert!(matches!(*modifier.expression, Expression::String(ref value) if value == "\"active\""));
    assert!(matches!(*modifier.condition, Expression::MemberAccess(_)));
    assert!(!Expression::Modifier(modifier).is_format_candidate());
}

#[test]
fn parses_modifier_unless_after_call_without_formatting_it() {
    let program = parse(r#"render(partial: "empty") unless hidden?"#).unwrap();

    let Expression::Modifier(modifier) = single_statement(&program) else {
        panic!("expected modifier expression");
    };

    assert_eq!(modifier.keyword, ModifierKeyword::Unless);
    assert!(matches!(*modifier.expression, Expression::Call(_)));
    assert!(matches!(*modifier.condition, Expression::Identifier(ref value) if value == "hidden?"));
}

#[test]
fn parses_safe_navigation_member_access() {
    let program = parse("user&.profile&.name").unwrap();

    let Expression::MemberAccess(name_access) = single_statement(&program) else {
        panic!("expected member access");
    };

    assert_eq!(name_access.operator, MemberOperator::SafeNavigation);
    assert_eq!(name_access.member, "name");

    let Expression::MemberAccess(profile_access) = *name_access.receiver else {
        panic!("expected nested safe navigation");
    };

    assert_eq!(profile_access.operator, MemberOperator::SafeNavigation);
    assert_eq!(profile_access.member, "profile");
}

#[test]
fn parses_index_access() {
    let program = parse("params[:id]").unwrap();

    let Expression::Index(index) = single_statement(&program) else {
        panic!("expected index access");
    };

    assert!(matches!(*index.receiver, Expression::Identifier(ref value) if value == "params"));
    assert_eq!(index.arguments.len(), 1);
    assert!(matches!(index.arguments[0], Expression::Symbol(ref value) if value == ":id"));
}

#[test]
fn parses_chained_safe_navigation_and_index_access() {
    let program = parse(r#"user&.profile[:name]"#).unwrap();

    let Expression::Index(index) = single_statement(&program) else {
        panic!("expected index access");
    };

    assert!(matches!(*index.receiver, Expression::MemberAccess(_)));
    assert!(matches!(index.arguments[0], Expression::Symbol(ref value) if value == ":name"));
}

#[test]
fn parses_ternary_expression_without_formatting_it() {
    let program = parse(r#"user.admin? ? "admin" : "user""#).unwrap();

    let Expression::Ternary(ternary) = single_statement(&program) else {
        panic!("expected ternary expression");
    };

    assert!(matches!(*ternary.condition, Expression::MemberAccess(_)));
    assert!(matches!(*ternary.truthy, Expression::String(ref value) if value == "\"admin\""));
    assert!(matches!(*ternary.falsy, Expression::String(ref value) if value == "\"user\""));
    assert!(!Expression::Ternary(ternary).is_format_candidate());
}

#[test]
fn parses_grouped_ternary_expression() {
    let program = parse(r#"(user.admin? ? "admin" : "user")"#).unwrap();

    let Expression::Grouped(grouped) = single_statement(&program) else {
        panic!("expected grouped ternary expression");
    };

    assert!(matches!(*grouped, Expression::Ternary(_)));
}

#[test]
fn parses_inclusive_range_expression() {
    let program = parse("1..10").unwrap();

    let Expression::Range(range) = single_statement(&program) else {
        panic!("expected range expression");
    };

    assert_eq!(range.operator, RangeOperator::Inclusive);
    assert!(matches!(*range.start, Expression::Literal(ref value) if value == "1"));
    assert!(matches!(*range.end, Expression::Literal(ref value) if value == "10"));
}

#[test]
fn parses_exclusive_range_expression() {
    let program = parse("start_date...end_date").unwrap();

    let Expression::Range(range) = single_statement(&program) else {
        panic!("expected range expression");
    };

    assert_eq!(range.operator, RangeOperator::Exclusive);
    assert!(matches!(*range.start, Expression::Identifier(ref value) if value == "start_date"));
    assert!(matches!(*range.end, Expression::Identifier(ref value) if value == "end_date"));
}

#[test]
fn parses_parenthesized_call_with_splat_arguments() {
    let program = parse("render(*partials, **options, &block)").unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.arguments.len(), 3);
    assert!(matches!(
        &call.arguments[0],
        Argument::Splat(Expression::Identifier(value)) if value == "partials"
    ));
    assert!(matches!(
        &call.arguments[1],
        Argument::KeywordSplat(Expression::Identifier(value)) if value == "options"
    ));
    assert!(matches!(
        &call.arguments[2],
        Argument::Block(Expression::Identifier(value)) if value == "block"
    ));
}

#[test]
fn parses_command_call_with_keyword_splat_argument() {
    let program = parse(r#"render partial: "card", **options"#).unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert_eq!(call.arguments.len(), 2);
    assert!(matches!(
        &call.arguments[1],
        Argument::KeywordSplat(Expression::Identifier(value)) if value == "options"
    ));
}

#[test]
fn parses_constant_path_expression() {
    let program = parse("Admin::User::Profile").unwrap();

    let Expression::ConstantPath(path) = single_statement(&program) else {
        panic!("expected constant path expression");
    };

    assert_eq!(path.parts, vec!["Admin", "User", "Profile"]);
}

#[test]
fn parses_constant_path_as_call_argument() {
    let program = parse("policy(Admin::User)").unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert!(matches!(
        &call.arguments[0],
        Argument::Positional(Expression::ConstantPath(path))
            if path.parts == vec!["Admin", "User"]
    ));
}

#[test]
fn parses_percent_literals_as_raw_literals() {
    let program =
        parse(r#"tag.div(class: %w[card active], data: { states: %i[draft live] })"#).unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert!(matches!(
        &call.arguments[0],
        Argument::Keyword { value: Expression::PercentLiteral(value), .. } if value == "%w[card active]"
    ));

    let Argument::Keyword {
        value: Expression::Hash(pairs),
        ..
    } = &call.arguments[1]
    else {
        panic!("expected keyword hash argument");
    };

    assert!(matches!(
        &pairs[0],
        HashPair::KeyValue { value: Expression::PercentLiteral(value), .. }
            if value == "%i[draft live]"
    ));
}

#[test]
fn parses_keyword_splat_hash_pairs() {
    let program = parse("tag.div(data: { **options })").unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    let Argument::Keyword {
        value: Expression::Hash(pairs),
        ..
    } = &call.arguments[0]
    else {
        panic!("expected keyword hash argument");
    };

    assert!(matches!(
        &pairs[0],
        HashPair::KeywordSplat(Expression::Identifier(value)) if value == "options"
    ));
}

#[test]
fn parses_tagged_percent_literals_with_custom_delimiters() {
    let program = parse(r#"path.match?(%r{/users/\d+})"#).unwrap();

    let Expression::Call(call) = single_statement(&program) else {
        panic!("expected call expression");
    };

    assert!(matches!(
        &call.arguments[0],
        Argument::Positional(Expression::PercentLiteral(value)) if value == r#"%r{/users/\d+}"#
    ));
}

fn single_statement(program: &super::ast::Program) -> Expression {
    assert_eq!(program.statements.len(), 1);
    program.statements[0].clone()
}
