#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub indent_html: bool,
    pub indent_style: IndentStyle,
    pub indent_width: usize,
    pub line_width: usize,
    pub line_ending: LineEnding,
    pub trailing_newline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndentStyle {
    Space,
    Tab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_html: true,
            indent_style: IndentStyle::Space,
            indent_width: 2,
            line_width: 80,
            line_ending: LineEnding::Lf,
            trailing_newline: true,
        }
    }
}

impl LineEnding {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }
}
