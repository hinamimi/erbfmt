#[derive(Debug)]
pub enum Token {
    Html(String),
    Erb(String),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    vec![Token::Html(input.to_string())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_html() {
        let tokens = tokenize("<div>Hello</div>");

        assert_eq!(tokens.len(), 1);
    }
}
