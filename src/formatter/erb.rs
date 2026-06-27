use super::ruby_wrap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ErbTagMarker {
    Code,
    Output,
}

impl ErbTagMarker {
    pub(super) fn from_output(output: bool) -> Self {
        if output { Self::Output } else { Self::Code }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Code => "<%",
            Self::Output => "<%=",
        }
    }
}

pub(super) fn format_erb_tag_inline(marker: ErbTagMarker, code: &str) -> String {
    if code.is_empty() {
        return format!("{} %>", marker.as_str());
    }

    format!("{} {} %>", marker.as_str(), code.trim())
}

pub(super) fn format_erb_comment(comment: &str) -> String {
    let comment = comment.trim();

    if comment.is_empty() {
        "<%# %>".to_string()
    } else {
        format!("<%# {comment} %>")
    }
}

fn normalized_erb_code_lines(code: &str) -> Vec<String> {
    let lines = trim_blank_edges(code.lines().collect());
    let common_indent = common_erb_code_indent(&lines);

    lines
        .into_iter()
        .map(|line| {
            strip_leading_whitespace(line, common_indent)
                .trim_end()
                .to_string()
        })
        .collect()
}

pub(super) fn formatted_erb_code_lines(code: &str) -> Vec<String> {
    if let Some(lines) = ruby_wrap::fold_command_call(code) {
        return lines;
    }

    normalized_erb_code_lines(code)
}

fn common_erb_code_indent(lines: &[&str]) -> usize {
    let non_empty_lines = lines.iter().copied().filter(|line| !line.trim().is_empty());

    if lines
        .first()
        .is_some_and(|line| leading_whitespace_count(line) == 0)
    {
        let skipped_first = lines
            .iter()
            .copied()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .map(leading_whitespace_count)
            .min();

        if let Some(indent) = skipped_first {
            return indent;
        }
    }

    non_empty_lines
        .map(leading_whitespace_count)
        .min()
        .unwrap_or(0)
}

fn trim_blank_edges(mut lines: Vec<&str>) -> Vec<&str> {
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }

    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }

    lines
}

fn leading_whitespace_count(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn strip_leading_whitespace(line: &str, count: usize) -> &str {
    if count == 0 {
        return line;
    }

    for (stripped, (index, ch)) in line.char_indices().enumerate() {
        if stripped == count || !ch.is_whitespace() {
            return &line[index..];
        }
    }

    ""
}
