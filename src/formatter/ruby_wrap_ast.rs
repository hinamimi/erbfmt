use crate::ruby_ast::{Argument, Call, Expression, HashKey, HashPair, Modifier, Program};

pub(super) fn foldable_call(program: &Program) -> Option<&Call> {
    if program.statements.len() != 1 {
        return None;
    }

    let call = match &program.statements[0] {
        Expression::Call(call) => call,
        Expression::Modifier(modifier) => foldable_modifier_call(modifier)?,
        _ => return None,
    };

    if call.arguments.iter().all(argument_is_foldable) {
        Some(call)
    } else {
        None
    }
}

fn foldable_modifier_call(modifier: &Modifier) -> Option<&Call> {
    if !expression_is_foldable(&modifier.condition) {
        return None;
    }

    match modifier.expression.as_ref() {
        Expression::Call(call) => Some(call),
        _ => None,
    }
}

fn argument_is_foldable(argument: &Argument) -> bool {
    match argument {
        Argument::Block(expression)
        | Argument::KeywordSplat(expression)
        | Argument::Positional(expression)
        | Argument::Splat(expression) => expression_is_foldable(expression),
        Argument::HashPair { key, value } => {
            expression_is_foldable(key) && expression_is_foldable(value)
        }
        Argument::Keyword { value, .. } => expression_is_foldable(value),
    }
}

pub(super) fn argument_contains_foldable_keyword_hash(argument: &Argument) -> bool {
    matches!(
        argument,
        Argument::Keyword {
            value: Expression::Hash(pairs),
            ..
        } if pairs.len() >= 2
    )
}

pub(super) fn expression_is_foldable(expression: &Expression) -> bool {
    match expression {
        Expression::Assignment(_) | Expression::Modifier(_) => false,
        Expression::Array(values) => values.iter().all(expression_is_foldable),
        Expression::Binary(binary) => {
            expression_is_foldable(&binary.left) && expression_is_foldable(&binary.right)
        }
        Expression::Call(call) => call.arguments.iter().all(argument_is_foldable),
        Expression::Grouped(expression) => expression_is_foldable(expression),
        Expression::Unary(unary) => expression_is_foldable(&unary.expression),
        Expression::Index(index) => {
            expression_is_foldable(&index.receiver)
                && index.arguments.iter().all(expression_is_foldable)
        }
        Expression::Hash(pairs) => pairs.iter().all(hash_pair_is_foldable),
        Expression::MemberAccess(access) => expression_is_foldable(&access.receiver),
        Expression::MethodCall(call) => {
            expression_is_foldable(&call.receiver)
                && call.arguments.iter().all(argument_is_foldable)
                && call
                    .block
                    .as_ref()
                    .is_none_or(|block| block.params.iter().all(|param| !param.is_empty()))
        }
        Expression::Range(range) => {
            expression_is_foldable(&range.start) && expression_is_foldable(&range.end)
        }
        Expression::Ternary(ternary) => {
            expression_is_foldable(&ternary.condition)
                && expression_is_foldable(&ternary.truthy)
                && expression_is_foldable(&ternary.falsy)
        }
        Expression::Identifier(_)
        | Expression::ConstantPath(_)
        | Expression::Literal(_)
        | Expression::PercentLiteral(_)
        | Expression::String(_)
        | Expression::Symbol(_) => true,
    }
}

fn hash_pair_is_foldable(pair: &HashPair) -> bool {
    match pair {
        HashPair::KeyValue { key, value } => {
            hash_key_is_foldable(key) && expression_is_foldable(value)
        }
        HashPair::KeywordSplat(expression) => expression_is_foldable(expression),
    }
}

fn hash_key_is_foldable(key: &HashKey) -> bool {
    match key {
        HashKey::Label(_) => true,
        HashKey::Expression(expression) => expression_is_foldable(expression),
    }
}
