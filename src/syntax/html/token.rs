#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    Text(String),
    OpenTag(HtmlTag),
    CloseTag(HtmlTag),
    SelfClosingTag(HtmlTag),
    VoidTag(HtmlTag),
    Comment(String),
    Doctype(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTag {
    pub name: String,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedHtmlToken {
    pub token: HtmlToken,
    pub span: HtmlSpan,
}

pub(super) fn spanned_html_token(start: usize, end: usize, token: HtmlToken) -> SpannedHtmlToken {
    SpannedHtmlToken {
        token,
        span: HtmlSpan { start, end },
    }
}
