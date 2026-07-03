use super::ast::{
    Argument, Assignment, AssignmentOperator, Binary, BinaryOperator, Block, Call, CallStyle,
    ConstantPath, Expression, HashKey, HashPair, Index, MemberAccess, MemberOperator, Modifier,
    ModifierKeyword, Program, Range, RangeOperator, Ternary, Unary, UnaryOperator,
};
use super::error::RubyAstError;
use super::lexer::{Token, TokenKind, tokenize};

pub fn parse(source: &str) -> Result<Program, RubyAstError> {
    let statements = split_top_level_statements(source.trim())?
        .into_iter()
        .map(parse_statement)
        .collect::<Result<Vec<_>, _>>()?;

    if statements.is_empty() {
        return Err(RubyAstError::new("expected Ruby statement", 0));
    }

    Ok(Program { statements })
}

fn parse_statement(source: &str) -> Result<Expression, RubyAstError> {
    let (source, block) = split_do_block(source.trim())?;

    if let Some((target, operator, value)) = split_assignment(source)? {
        if block.is_some() {
            return Err(RubyAstError::new(
                "assignment expressions cannot have a do block suffix",
                source.len(),
            ));
        }

        let value = parse_statement(value)?;

        return Ok(Expression::Assignment(Assignment {
            target: target.to_string(),
            operator,
            value: Box::new(value),
        }));
    }

    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    let expression = parser.parse_expression_with_optional_block(block)?;
    parser.expect_end()?;

    Ok(expression)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse_expression_with_optional_block(
        &mut self,
        block: Option<Block>,
    ) -> Result<Expression, RubyAstError> {
        let expression = if self.is_parenthesized_call_start() || self.is_command_call_start() {
            self.parse_call(block).map(Expression::Call)?
        } else {
            if block.is_some() {
                return Err(self.error("do blocks require a method call"));
            }

            self.parse_ternary_expression()?
        };

        self.parse_optional_modifier(expression)
    }

    fn parse_modifier_expression(&mut self) -> Result<Expression, RubyAstError> {
        let expression = self.parse_ternary_expression()?;

        self.parse_optional_modifier(expression)
    }

    fn parse_optional_modifier(
        &mut self,
        expression: Expression,
    ) -> Result<Expression, RubyAstError> {
        let Some(keyword) = self.peek_modifier_keyword() else {
            return Ok(expression);
        };

        self.advance();
        let condition = self.parse_ternary_expression()?;

        Ok(Expression::Modifier(Modifier {
            expression: Box::new(expression),
            keyword,
            condition: Box::new(condition),
        }))
    }

    fn parse_ternary_expression(&mut self) -> Result<Expression, RubyAstError> {
        let condition = self.parse_expression(0)?;

        if !self.consume_punct('?') {
            return Ok(condition);
        }

        let truthy = self.parse_ternary_expression()?;
        self.expect_punct(':')?;
        let falsy = self.parse_ternary_expression()?;

        Ok(Expression::Ternary(Ternary {
            condition: Box::new(condition),
            truthy: Box::new(truthy),
            falsy: Box::new(falsy),
        }))
    }

    fn parse_expression(&mut self, minimum_precedence: u8) -> Result<Expression, RubyAstError> {
        let mut expression = self.parse_unary_expression()?;

        loop {
            if let Some((operator, precedence)) = self.peek_range_operator() {
                if precedence < minimum_precedence {
                    break;
                }

                self.advance();
                let end = self.parse_expression(precedence + 1)?;
                expression = Expression::Range(Range {
                    start: Box::new(expression),
                    operator,
                    end: Box::new(end),
                });
                continue;
            }

            let Some((operator, precedence)) = self.peek_binary_operator() else {
                break;
            };

            if precedence < minimum_precedence {
                break;
            }

            self.advance();
            let right = self.parse_expression(precedence + 1)?;
            expression = Expression::Binary(Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expression)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, RubyAstError> {
        if let Some(operator) = self.peek_unary_operator() {
            self.advance();
            let expression = self.parse_unary_expression()?;

            return Ok(Expression::Unary(Unary {
                operator,
                expression: Box::new(expression),
            }));
        }

        self.parse_postfix_expression()
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression, RubyAstError> {
        let mut expression = self.parse_primary_expression()?;

        loop {
            if self.consume_punct('.') {
                expression = Expression::MemberAccess(MemberAccess {
                    receiver: Box::new(expression),
                    operator: MemberOperator::Dot,
                    member: self.expect_identifier()?,
                });
                continue;
            }

            if self.consume_safe_navigation() {
                expression = Expression::MemberAccess(MemberAccess {
                    receiver: Box::new(expression),
                    operator: MemberOperator::SafeNavigation,
                    member: self.expect_identifier()?,
                });
                continue;
            }

            if self.consume_double_colon() {
                expression = append_constant_path(expression, self.expect_identifier()?);
                continue;
            }

            if self.consume_punct('[') {
                let arguments = self.parse_index_arguments()?;
                self.expect_punct(']')?;
                expression = Expression::Index(Index {
                    receiver: Box::new(expression),
                    arguments,
                });
                continue;
            }

            break;
        }

        Ok(expression)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, RubyAstError> {
        if self.is_parenthesized_call_start() {
            return self.parse_call(None).map(Expression::Call);
        }

        if self.consume_punct('(') {
            let expression = self.parse_ternary_expression()?;
            self.expect_punct(')')?;

            return Ok(Expression::Grouped(Box::new(expression)));
        }

        if self.check_punct('[') {
            return self.parse_array();
        }

        if self.check_punct('{') {
            return self.parse_hash();
        }

        match self.advance().map(|token| token.kind.clone()) {
            Some(TokenKind::Identifier(value)) => Ok(Expression::Identifier(value)),
            Some(TokenKind::Number(value)) => Ok(Expression::Literal(value)),
            Some(TokenKind::PercentLiteral(value)) => Ok(Expression::PercentLiteral(value)),
            Some(TokenKind::String(value)) => Ok(Expression::String(value)),
            Some(TokenKind::Symbol(value)) => Ok(Expression::Symbol(value)),
            Some(kind) => Err(self.error(format!("expected Ruby value, found {kind:?}"))),
            None => Err(self.error("expected Ruby value")),
        }
    }

    fn parse_call(&mut self, block: Option<Block>) -> Result<Call, RubyAstError> {
        let callee = self.parse_callee()?;

        if self.consume_punct('(') {
            let arguments = self.parse_argument_list_until(')')?;
            self.expect_punct(')')?;

            return Ok(Call {
                callee,
                arguments,
                style: CallStyle::Parenthesized,
                block,
            });
        }

        let arguments = self.parse_remaining_arguments()?;
        if arguments.is_empty() {
            return Err(self.error("command calls require at least one argument"));
        }

        Ok(Call {
            callee,
            arguments,
            style: CallStyle::Command,
            block,
        })
    }

    fn parse_callee(&mut self) -> Result<String, RubyAstError> {
        let mut callee = self.expect_identifier()?;

        while self.consume_punct('.') || self.consume_double_colon() {
            let separator = match self.previous().map(|token| &token.kind) {
                Some(TokenKind::Punct('.')) => ".",
                Some(TokenKind::DoubleColon) => "::",
                _ => unreachable!("callee separator was just consumed"),
            };
            let method = self.expect_identifier()?;
            callee.push_str(separator);
            callee.push_str(&method);
        }

        Ok(callee)
    }

    fn parse_remaining_arguments(&mut self) -> Result<Vec<Argument>, RubyAstError> {
        let mut arguments = Vec::new();

        while !self.is_at_end() {
            arguments.push(self.parse_argument()?);

            if !self.consume_punct(',') {
                break;
            }
        }

        Ok(arguments)
    }

    fn parse_argument_list_until(&mut self, close: char) -> Result<Vec<Argument>, RubyAstError> {
        let mut arguments = Vec::new();

        if self.check_punct(close) {
            return Ok(arguments);
        }

        loop {
            arguments.push(self.parse_argument()?);

            if self.check_punct(close) {
                break;
            }

            self.expect_punct(',')?;

            if self.check_punct(close) {
                break;
            }
        }

        Ok(arguments)
    }

    fn parse_argument(&mut self) -> Result<Argument, RubyAstError> {
        if self.consume_operator("*") {
            return Ok(Argument::Splat(self.parse_modifier_expression()?));
        }

        if self.consume_operator("**") {
            return Ok(Argument::KeywordSplat(self.parse_modifier_expression()?));
        }

        if self.consume_operator("&") {
            return Ok(Argument::Block(self.parse_modifier_expression()?));
        }

        if let Some(name) = self.peek_identifier_before_colon() {
            self.advance();
            self.expect_punct(':')?;
            let value = self.parse_modifier_expression()?;
            return Ok(Argument::Keyword { name, value });
        }

        let expression = self.parse_modifier_expression()?;

        if self.consume_operator("=>") {
            let value = self.parse_modifier_expression()?;
            return Ok(Argument::HashPair {
                key: expression,
                value,
            });
        }

        Ok(Argument::Positional(expression))
    }

    fn parse_array(&mut self) -> Result<Expression, RubyAstError> {
        self.expect_punct('[')?;
        let mut values = Vec::new();

        if !self.check_punct(']') {
            loop {
                values.push(self.parse_modifier_expression()?);

                if self.check_punct(']') {
                    break;
                }

                self.expect_punct(',')?;

                if self.check_punct(']') {
                    break;
                }
            }
        }

        self.expect_punct(']')?;
        Ok(Expression::Array(values))
    }

    fn parse_hash(&mut self) -> Result<Expression, RubyAstError> {
        self.expect_punct('{')?;
        let mut pairs = Vec::new();

        if !self.check_punct('}') {
            loop {
                pairs.push(self.parse_hash_pair()?);

                if self.check_punct('}') {
                    break;
                }

                self.expect_punct(',')?;

                if self.check_punct('}') {
                    break;
                }
            }
        }

        self.expect_punct('}')?;
        Ok(Expression::Hash(pairs))
    }

    fn parse_hash_pair(&mut self) -> Result<HashPair, RubyAstError> {
        if self.consume_operator("**") {
            return Ok(HashPair::KeywordSplat(self.parse_modifier_expression()?));
        }

        if let Some(label) = self.peek_identifier_before_colon() {
            self.advance();
            self.expect_punct(':')?;
            let value = self.parse_modifier_expression()?;

            return Ok(HashPair::KeyValue {
                key: HashKey::Label(label),
                value,
            });
        }

        let key = self.parse_modifier_expression()?;
        self.expect_operator("=>")?;
        let value = self.parse_modifier_expression()?;

        Ok(HashPair::KeyValue {
            key: HashKey::Expression(key),
            value,
        })
    }

    fn is_parenthesized_call_start(&self) -> bool {
        self.index_after_callee(self.position).is_some_and(|index| {
            matches!(
                self.tokens.get(index).map(|token| &token.kind),
                Some(TokenKind::Punct('('))
            )
        })
    }

    fn is_command_call_start(&self) -> bool {
        let Some(index) = self.index_after_callee(self.position) else {
            return false;
        };

        matches!(
            self.tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
                | Some(TokenKind::Number(_))
                | Some(TokenKind::PercentLiteral(_))
                | Some(TokenKind::String(_))
                | Some(TokenKind::Symbol(_))
                | Some(TokenKind::Punct('{'))
        )
    }

    fn index_after_callee(&self, start: usize) -> Option<usize> {
        if !matches!(
            self.tokens.get(start).map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) {
            return None;
        }

        let mut index = start + 1;

        while matches!(
            self.tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Punct('.')) | Some(TokenKind::DoubleColon)
        ) {
            index += 1;

            if !matches!(
                self.tokens.get(index).map(|token| &token.kind),
                Some(TokenKind::Identifier(_))
            ) {
                return None;
            }

            index += 1;
        }

        Some(index)
    }

    fn expect_identifier(&mut self) -> Result<String, RubyAstError> {
        match self.advance().map(|token| token.kind.clone()) {
            Some(TokenKind::Identifier(value)) if !is_keyword(&value) => Ok(value),
            Some(TokenKind::Identifier(value)) => {
                Err(self.error(format!("Ruby keyword `{value}` is not a call name")))
            }
            Some(kind) => Err(self.error(format!("expected Ruby identifier, found {kind:?}"))),
            None => Err(self.error("expected Ruby identifier")),
        }
    }

    fn parse_index_arguments(&mut self) -> Result<Vec<Expression>, RubyAstError> {
        let mut arguments = Vec::new();

        if self.check_punct(']') {
            return Ok(arguments);
        }

        loop {
            arguments.push(self.parse_modifier_expression()?);

            if self.check_punct(']') {
                break;
            }

            self.expect_punct(',')?;

            if self.check_punct(']') {
                break;
            }
        }

        Ok(arguments)
    }

    fn peek_binary_operator(&self) -> Option<(BinaryOperator, u8)> {
        let Some(TokenKind::Operator(operator)) = self.peek().map(|token| &token.kind) else {
            return None;
        };

        let (operator, precedence) = match operator.as_str() {
            "||" => (BinaryOperator::Or, 1),
            "&&" => (BinaryOperator::And, 2),
            "==" => (BinaryOperator::Equal, 3),
            "!=" => (BinaryOperator::NotEqual, 3),
            "<" => (BinaryOperator::LessThan, 4),
            "<=" => (BinaryOperator::LessThanOrEqual, 4),
            ">" => (BinaryOperator::GreaterThan, 4),
            ">=" => (BinaryOperator::GreaterThanOrEqual, 4),
            "+" => (BinaryOperator::Add, 5),
            "-" => (BinaryOperator::Subtract, 5),
            "*" => (BinaryOperator::Multiply, 6),
            "/" => (BinaryOperator::Divide, 6),
            "%" => (BinaryOperator::Modulo, 6),
            _ => return None,
        };

        Some((operator, precedence))
    }

    fn peek_range_operator(&self) -> Option<(RangeOperator, u8)> {
        let Some(TokenKind::Operator(operator)) = self.peek().map(|token| &token.kind) else {
            return None;
        };

        match operator.as_str() {
            ".." => Some((RangeOperator::Inclusive, 3)),
            "..." => Some((RangeOperator::Exclusive, 3)),
            _ => None,
        }
    }

    fn peek_unary_operator(&self) -> Option<UnaryOperator> {
        match self.peek().map(|token| &token.kind) {
            Some(TokenKind::Operator(operator)) if operator == "!" => Some(UnaryOperator::Not),
            Some(TokenKind::Operator(operator)) if operator == "-" => Some(UnaryOperator::Negate),
            _ => None,
        }
    }

    fn peek_modifier_keyword(&self) -> Option<ModifierKeyword> {
        match self.peek().map(|token| &token.kind) {
            Some(TokenKind::Identifier(keyword)) if keyword == "if" => Some(ModifierKeyword::If),
            Some(TokenKind::Identifier(keyword)) if keyword == "unless" => {
                Some(ModifierKeyword::Unless)
            }
            _ => None,
        }
    }

    fn peek_identifier_before_colon(&self) -> Option<String> {
        let current = self.tokens.get(self.position)?;
        let next = self.tokens.get(self.position + 1)?;

        match (&current.kind, &next.kind) {
            (TokenKind::Identifier(name), TokenKind::Punct(':')) => Some(name.clone()),
            _ => None,
        }
    }

    fn consume_punct(&mut self, punct: char) -> bool {
        if self.check_punct(punct) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn consume_double_colon(&mut self) -> bool {
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::DoubleColon)
        ) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn consume_safe_navigation(&mut self) -> bool {
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::SafeNavigation)
        ) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn check_punct(&self, punct: char) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Punct(value)) if *value == punct
        )
    }

    fn expect_punct(&mut self, punct: char) -> Result<(), RubyAstError> {
        if self.consume_punct(punct) {
            Ok(())
        } else {
            Err(self.error(format!("expected `{punct}`")))
        }
    }

    fn expect_operator(&mut self, operator: &str) -> Result<(), RubyAstError> {
        match self.advance().map(|token| token.kind.clone()) {
            Some(TokenKind::Operator(value)) if value == operator => Ok(()),
            Some(kind) => Err(self.error(format!("expected `{operator}`, found {kind:?}"))),
            None => Err(self.error(format!("expected `{operator}`"))),
        }
    }

    fn consume_operator(&mut self, operator: &str) -> bool {
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Operator(value)) if value == operator
        ) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expect_end(&self) -> Result<(), RubyAstError> {
        if self.is_at_end() {
            Ok(())
        } else {
            Err(self.error("unexpected Ruby tokens after expression"))
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += usize::from(token.is_some());
        token
    }

    fn previous(&self) -> Option<&Token> {
        self.position
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn error(&self, message: impl Into<String>) -> RubyAstError {
        let offset = self.peek().map(|token| token.offset).unwrap_or_default();
        RubyAstError::new(message, offset)
    }
}

fn split_do_block(source: &str) -> Result<(&str, Option<Block>), RubyAstError> {
    let mut scanner = Scanner::default();
    let mut block_start = None;

    for (offset, ch) in source.char_indices() {
        if scanner.is_top_level()
            && ch.is_whitespace()
            && source[offset..].trim_start().starts_with("do")
        {
            let leading = source[offset..].len() - source[offset..].trim_start().len();
            let candidate = offset + leading;
            let after_do = &source[candidate + "do".len()..];

            if after_do.is_empty() || after_do.chars().next().is_some_and(char::is_whitespace) {
                block_start = Some(candidate);
            }
        }

        if scanner.is_top_level() && ch.is_whitespace() {
            let leading = source[offset..].len() - source[offset..].trim_start().len();
            let candidate = offset + leading;
            let before = source[..candidate].trim_end();
            let suffix = source[candidate..].trim();

            if before.ends_with(')')
                && suffix.starts_with('{')
                && is_balanced_brace_block_suffix(suffix)
            {
                block_start = Some(candidate);
            }
        }

        scanner.consume(ch)?;
    }

    let Some(block_start) = block_start else {
        return Ok((source, None));
    };

    let call = source[..block_start].trim_end();
    let suffix = source[block_start..].trim();
    let block = parse_block_suffix(suffix, block_start)?;

    Ok((call, Some(block)))
}

fn split_top_level_statements(source: &str) -> Result<Vec<&str>, RubyAstError> {
    let mut scanner = Scanner::default();
    let mut statements = Vec::new();
    let mut start = 0;

    for (offset, ch) in source.char_indices() {
        if scanner.is_top_level() && matches!(ch, ';' | '\n') {
            let statement = source[start..offset].trim();

            if !statement.is_empty() {
                statements.push(statement);
            }

            start = offset + ch.len_utf8();
            continue;
        }

        scanner.consume(ch)?;
    }

    let statement = source[start..].trim();
    if !statement.is_empty() {
        statements.push(statement);
    }

    Ok(statements)
}

fn split_assignment(
    source: &str,
) -> Result<Option<(&str, AssignmentOperator, &str)>, RubyAstError> {
    let mut scanner = Scanner::default();

    for (offset, ch) in source.char_indices() {
        if scanner.is_top_level()
            && let Some((raw_operator, operator)) = assignment_operator_at(source, offset)
        {
            let target = source[..offset].trim();
            let value = source[offset + raw_operator.len()..].trim();

            if target.is_empty() || value.is_empty() {
                return Err(RubyAstError::new("invalid Ruby assignment", offset));
            }

            return Ok(Some((target, operator, value)));
        }

        scanner.consume(ch)?;
    }

    Ok(None)
}

fn assignment_operator_at(
    source: &str,
    offset: usize,
) -> Option<(&'static str, AssignmentOperator)> {
    let remaining = &source[offset..];

    for (raw, operator) in [
        ("||=", AssignmentOperator::Or),
        ("&&=", AssignmentOperator::And),
        ("+=", AssignmentOperator::Add),
        ("-=", AssignmentOperator::Subtract),
        ("*=", AssignmentOperator::Multiply),
        ("/=", AssignmentOperator::Divide),
        ("%=", AssignmentOperator::Modulo),
        ("=", AssignmentOperator::Assign),
    ] {
        if remaining.starts_with(raw) && is_assignment_boundary(source, offset, raw.len()) {
            return Some((raw, operator));
        }
    }

    None
}

fn is_assignment_boundary(source: &str, offset: usize, len: usize) -> bool {
    let before = source[..offset].chars().next_back();
    let after = source[offset + len..].chars().next();

    if len == 1 {
        !matches!(
            before,
            Some('=' | '!' | '<' | '>' | '&' | '|' | '+' | '-' | '*' | '/' | '%')
        ) && !matches!(after, Some('=' | '>'))
    } else {
        true
    }
}

fn parse_block_suffix(suffix: &str, offset: usize) -> Result<Block, RubyAstError> {
    if suffix.starts_with('{') {
        return parse_brace_block_suffix(suffix, offset);
    }

    let rest = suffix
        .strip_prefix("do")
        .ok_or_else(|| RubyAstError::new("expected `do` block suffix", offset))?
        .trim();

    if rest.is_empty() {
        return Ok(Block { params: Vec::new() });
    }

    if !rest.starts_with('|') || !rest.ends_with('|') {
        return Err(RubyAstError::new(
            "only `do |param|` block suffixes are supported",
            offset,
        ));
    }

    let params = rest[1..rest.len() - 1]
        .split(',')
        .map(str::trim)
        .filter(|param| !param.is_empty())
        .map(ToString::to_string)
        .collect();

    Ok(Block { params })
}

fn parse_brace_block_suffix(suffix: &str, offset: usize) -> Result<Block, RubyAstError> {
    if !is_balanced_brace_block_suffix(suffix) {
        return Err(RubyAstError::new("unbalanced `{` block suffix", offset));
    }

    let body = suffix[1..suffix.len() - 1].trim();
    if !body.starts_with('|') {
        return Ok(Block { params: Vec::new() });
    }

    let Some(end) = body[1..].find('|').map(|index| index + 1) else {
        return Err(RubyAstError::new(
            "unterminated `{ |param| }` block parameters",
            offset,
        ));
    };

    let params = body[1..end]
        .split(',')
        .map(str::trim)
        .filter(|param| !param.is_empty())
        .map(ToString::to_string)
        .collect();

    Ok(Block { params })
}

fn is_balanced_brace_block_suffix(suffix: &str) -> bool {
    if !suffix.starts_with('{') || !suffix.ends_with('}') {
        return false;
    }

    let mut scanner = Scanner::default();

    for (offset, ch) in suffix.char_indices() {
        if offset > 0 && scanner.is_top_level() {
            return false;
        }

        if scanner.consume(ch).is_err() {
            return false;
        }
    }

    scanner.is_top_level()
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "alias"
            | "and"
            | "begin"
            | "break"
            | "case"
            | "class"
            | "def"
            | "do"
            | "else"
            | "elsif"
            | "end"
            | "ensure"
            | "false"
            | "for"
            | "if"
            | "in"
            | "module"
            | "next"
            | "nil"
            | "not"
            | "or"
            | "redo"
            | "rescue"
            | "return"
            | "self"
            | "super"
            | "then"
            | "true"
            | "unless"
            | "until"
            | "when"
            | "while"
            | "yield"
    )
}

fn append_constant_path(expression: Expression, part: String) -> Expression {
    match expression {
        Expression::Identifier(name) => Expression::ConstantPath(ConstantPath {
            parts: vec![name, part],
        }),
        Expression::ConstantPath(mut path) => {
            path.parts.push(part);
            Expression::ConstantPath(path)
        }
        expression => Expression::MemberAccess(MemberAccess {
            receiver: Box::new(expression),
            operator: MemberOperator::Dot,
            member: part,
        }),
    }
}

#[derive(Default)]
struct Scanner {
    stack: Vec<char>,
    string: Option<char>,
    escaped: bool,
}

impl Scanner {
    fn is_top_level(&self) -> bool {
        self.string.is_none() && self.stack.is_empty()
    }

    fn consume(&mut self, ch: char) -> Result<(), RubyAstError> {
        if let Some(quote) = self.string {
            if self.escaped {
                self.escaped = false;
                return Ok(());
            }

            if ch == '\\' {
                self.escaped = true;
                return Ok(());
            }

            if ch == quote {
                self.string = None;
            }

            return Ok(());
        }

        match ch {
            '\'' | '"' => self.string = Some(ch),
            '(' | '[' | '{' => self.stack.push(ch),
            ')' if self.stack.pop() != Some('(') => {
                return Err(RubyAstError::new("unbalanced `)`", 0));
            }
            ']' if self.stack.pop() != Some('[') => {
                return Err(RubyAstError::new("unbalanced `]`", 0));
            }
            '}' if self.stack.pop() != Some('{') => {
                return Err(RubyAstError::new("unbalanced `}`", 0));
            }
            _ => {}
        }

        Ok(())
    }
}
