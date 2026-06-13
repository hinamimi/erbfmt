use crate::{lexer, parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
}

pub fn lint(input: &str) -> Vec<Diagnostic> {
    let tokens = match lexer::tokenize(input) {
        Ok(tokens) => tokens,
        Err(error) => {
            return vec![Diagnostic {
                message: error.to_string(),
            }];
        }
    };

    match parser::parse(&tokens) {
        Ok(_) => Vec::new(),
        Err(error) => {
            vec![Diagnostic {
                message: error.to_string(),
            }]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_no_diagnostics_for_valid_template() {
        let diagnostics = lint("<% if user %>\n<p>Hello</p>\n<% end %>\n");

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_unterminated_erb_tag() {
        let diagnostics = lint("<div><% if user");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "unterminated ERB tag at byte 5".to_string()
            }]
        );
    }

    #[test]
    fn reports_unexpected_block_end() {
        let diagnostics = lint("<% end %>");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "unexpected ERB block end `end` at token 0".to_string()
            }]
        );
    }

    #[test]
    fn reports_unclosed_block() {
        let diagnostics = lint("<% if user %>\n<p>Hello</p>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "unclosed ERB block `if user` at token 0".to_string()
            }]
        );
    }
}
