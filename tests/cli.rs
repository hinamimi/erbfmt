use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

const UNFORMATTED: &str = "<div>\n<p>Hello</p>\n</div>\n";
const FORMATTED: &str = "<div>\n  <p>Hello</p>\n</div>\n";

#[test]
fn version_reports_crate_version() {
    let output = run(["--version"]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("erbfmt {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn help_describes_core_modes() {
    let output = run(["--help"]);

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(
        stdout.contains("Format and lint Ruby ERB templates"),
        "{stdout}"
    );
    assert!(stdout.contains("--write"), "{stdout}");
    assert!(stdout.contains("--check"), "{stdout}");
    assert!(stdout.contains("--lint"), "{stdout}");
    assert!(stdout.contains("--config"), "{stdout}");
    assert_eq!(stderr(&output), "");
}

#[test]
fn formats_single_file_to_stdout() {
    let dir = TestDir::new("stdout");
    let file = dir.write("input.html.erb", UNFORMATTED);

    let output = run([file.as_path()]);

    assert_success(&output);
    assert_eq!(stdout(&output), FORMATTED);
    assert_eq!(stderr(&output), "");
}

#[test]
fn write_formats_file_in_place() {
    let dir = TestDir::new("write");
    let file = dir.write("input.html.erb", UNFORMATTED);

    let output = run(["--write".as_ref(), file.as_path()]);

    assert_success(&output);
    assert_eq!(fs::read_to_string(&file).unwrap(), FORMATTED);
    assert_eq!(
        stdout(&output),
        format!("{}: wrote formatted file.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn check_passes_for_formatted_file() {
    let dir = TestDir::new("check_pass");
    let file = dir.write("input.html.erb", FORMATTED);

    let output = run(["--check".as_ref(), file.as_path()]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: file is formatted.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn check_fails_for_unformatted_file() {
    let dir = TestDir::new("check_fail");
    let file = dir.write("input.html.erb", UNFORMATTED);

    let output = run(["--check".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!("{}: file is not formatted.\n", file.display())
    );
}

#[test]
fn lint_passes_for_valid_file() {
    let dir = TestDir::new("lint_pass");
    let file = dir.write("input.html.erb", FORMATTED);

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn lint_fails_for_invalid_file() {
    let dir = TestDir::new("lint_fail");
    let file = dir.write("input.html.erb", "<% if show_empty_state %>\n<% end %>\n");

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: empty ERB control block `<% if show_empty_state %>` at line 1, column 1\n",
            file.display()
        )
    );
}

#[test]
fn lint_fails_for_empty_erb_code_tags() {
    let dir = TestDir::new("lint_empty_erb_code_tag");
    let file = dir.write("input.html.erb", "<p>Before</p>\n  <% %>\n  <%=   %>\n");

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: empty ERB code tag `<% %>` at line 2, column 3\n{}: empty ERB output tag `<%= %>` at line 3, column 3\n",
            file.display(),
            file.display()
        )
    );
}

#[test]
fn lint_fails_for_empty_erb_branches() {
    let dir = TestDir::new("lint_empty_erb_branch");
    let file = dir.write(
        "input.html.erb",
        "<% if current_user %>\n<p>Hello</p>\n<% else %>\n<% end %>\n",
    );

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: empty ERB branch `<% else %>` at line 3, column 1\n",
            file.display()
        )
    );
}

#[test]
fn lint_fails_for_html_rules() {
    let dir = TestDir::new("lint_html_rules");
    let file = dir.write(
        "input.html.erb",
        "<main>\n  <center>Legacy</center>\n  <div />\n</main>\n",
    );

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: deprecated HTML tag `<center>` at line 2, column 3\n{}: self-closing HTML tag `<div />` is not valid HTML5 at line 3, column 3\n",
            file.display(),
            file.display()
        )
    );
}

#[test]
fn lint_fails_for_duplicate_html_attributes() {
    let dir = TestDir::new("lint_duplicate_html_attributes");
    let file = dir.write(
        "input.html.erb",
        "<article class=\"card\" id=\"one\" class=\"wide\"></article>\n",
    );

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: duplicate HTML attribute `class` at line 1, column 32\n",
            file.display()
        )
    );
}

#[test]
fn lint_fails_for_invalid_html_nesting() {
    let dir = TestDir::new("lint_invalid_html_nesting");
    let file = dir.write(
        "input.html.erb",
        "<ul>\n  <div>Bad</div>\n</ul>\n<p>\n  <div>Bad</div>\n</p>\n<table>\n  <tr><div>Bad</div></tr>\n</table>\n",
    );

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: invalid HTML nesting: <ul> cannot have <div> as a direct child at line 2, column 3\n{}: invalid HTML nesting: <p> cannot contain <div> at line 5, column 3\n{}: invalid HTML nesting: <tr> cannot have <div> as a direct child at line 8, column 7\n",
            file.display(),
            file.display(),
            file.display()
        )
    );
}

#[test]
fn config_controls_formatter_indent_width() {
    let dir = TestDir::new("config_indent_width");
    let config = dir.write("erbfmt.json", r#"{"formatter":{"indentWidth":4}}"#);
    let file = dir.write("input.html.erb", UNFORMATTED);

    let output = run(["--config".as_ref(), config.as_path(), file.as_path()]);

    assert_success(&output);
    assert_eq!(stdout(&output), "<div>\n    <p>Hello</p>\n</div>\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_controls_formatter_line_width() {
    let dir = TestDir::new("config_line_width");
    let config = dir.write("erbfmt.json", r#"{"formatter":{"lineWidth":48}}"#);
    let file = dir.write(
        "input.html.erb",
        r#"<article class="card" data-user-id="<%= user.id %>" aria-label="Current user profile"><p>Hello</p></article>"#,
    );

    let output = run(["--config".as_ref(), config.as_path(), file.as_path()]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        "<article\n  class=\"card\"\n  data-user-id=\"<%= user.id %>\"\n  aria-label=\"Current user profile\"\n>\n  <p>Hello</p>\n</article>\n"
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_controls_erb_tag_line_width() {
    let dir = TestDir::new("config_erb_line_width");
    let config = dir.write("erbfmt.json", r#"{"formatter":{"lineWidth":60}}"#);
    let file = dir.write(
        "input.html.erb",
        r#"<%= link_to "Edit profile", edit_user_path(user), class: "button button--primary", data: { turbo_frame: "_top" } %>"#,
    );

    let output = run(["--config".as_ref(), config.as_path(), file.as_path()]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        "<%=\n  link_to \"Edit profile\", edit_user_path(user), class: \"button button--primary\", data: { turbo_frame: \"_top\" }\n%>\n"
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_html_indentation() {
    let dir = TestDir::new("config_indent_html_false");
    let config = dir.write("erbfmt.json", r#"{"formatter":{"indentHtml":false}}"#);
    let file = dir.write(
        "input.html.erb",
        "<% if user %>\n<ul>\n<li>Hello</li>\n</ul>\n<% end %>\n",
    );

    let output = run(["--config".as_ref(), config.as_path(), file.as_path()]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        "<% if user %>\n  <ul>\n  <li>Hello</li>\n  </ul>\n<% end %>\n"
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_formatter() {
    let dir = TestDir::new("config_formatter_disabled");
    let config = dir.write("erbfmt.json", r#"{"formatter":{"enabled":false}}"#);
    let file = dir.write("input.html.erb", UNFORMATTED);

    let output = run(["--config".as_ref(), config.as_path(), file.as_path()]);

    assert_success(&output);
    assert_eq!(stdout(&output), UNFORMATTED);
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_linter_rule() {
    let dir = TestDir::new("config_lint_rule_disabled");
    let config = dir.write(
        "erbfmt.json",
        r#"{"linter":{"rules":{"emptyErbControlBlock":"off"}}}"#,
    );
    let file = dir.write("input.html.erb", "<% if show_empty_state %>\n<% end %>\n");

    let output = run([
        "--lint".as_ref(),
        "--config".as_ref(),
        config.as_path(),
        file.as_path(),
    ]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_empty_erb_code_tag_rule() {
    let dir = TestDir::new("config_empty_erb_code_tag_disabled");
    let config = dir.write(
        "erbfmt.json",
        r#"{"linter":{"rules":{"emptyErbCodeTag":"off"}}}"#,
    );
    let file = dir.write("input.html.erb", "<% %>\n<%= %>\n");

    let output = run([
        "--lint".as_ref(),
        "--config".as_ref(),
        config.as_path(),
        file.as_path(),
    ]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_empty_erb_branch_rule() {
    let dir = TestDir::new("config_empty_erb_branch_disabled");
    let config = dir.write(
        "erbfmt.json",
        r#"{"linter":{"rules":{"emptyErbBranch":"off"}}}"#,
    );
    let file = dir.write(
        "input.html.erb",
        "<% if current_user %>\n<p>Hello</p>\n<% else %>\n<% end %>\n",
    );

    let output = run([
        "--lint".as_ref(),
        "--config".as_ref(),
        config.as_path(),
        file.as_path(),
    ]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_html_rules() {
    let dir = TestDir::new("config_html_rules_disabled");
    let config = dir.write(
        "erbfmt.json",
        r#"{"linter":{"rules":{"noDeprecatedHtmlTag":"off","noSelfClosingHtmlTag":"off"}}}"#,
    );
    let file = dir.write("input.html.erb", "<center><div /></center>\n");

    let output = run([
        "--lint".as_ref(),
        "--config".as_ref(),
        config.as_path(),
        file.as_path(),
    ]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_duplicate_html_attribute_rule() {
    let dir = TestDir::new("config_duplicate_html_attribute_disabled");
    let config = dir.write(
        "erbfmt.json",
        r#"{"linter":{"rules":{"noDuplicateHtmlAttribute":"off"}}}"#,
    );
    let file = dir.write("input.html.erb", r#"<div class="card" class="wide"></div>"#);

    let output = run([
        "--lint".as_ref(),
        "--config".as_ref(),
        config.as_path(),
        file.as_path(),
    ]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn config_can_disable_invalid_html_nesting_rule() {
    let dir = TestDir::new("config_invalid_html_nesting_disabled");
    let config = dir.write(
        "erbfmt.json",
        r#"{"linter":{"rules":{"noInvalidHtmlNesting":"off"}}}"#,
    );
    let file = dir.write("input.html.erb", "<ul><div>Bad</div></ul>\n");

    let output = run([
        "--lint".as_ref(),
        "--config".as_ref(),
        config.as_path(),
        file.as_path(),
    ]);

    assert_success(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", file.display())
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn lint_lexer_errors_include_line_and_column() {
    let dir = TestDir::new("lint_lex_location");
    let file = dir.write("input.html.erb", "<div>\n  <% if user");

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: unterminated ERB tag at line 2, column 3\n",
            file.display()
        )
    );
}

#[test]
fn lint_parser_errors_include_line_and_column() {
    let dir = TestDir::new("lint_parse_location");
    let file = dir.write("input.html.erb", "<p>Hello</p>\n<% end %>\n");

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: unexpected ERB block end `end` at line 2, column 1\n",
            file.display()
        )
    );
}

#[test]
fn lint_unexpected_html_close_errors_include_close_tag_location() {
    let dir = TestDir::new("lint_unexpected_html_close_location");
    let file = dir.write("input.html.erb", "<p>Hello</p>\n</div>\n");

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: unexpected HTML close tag `</div>` at line 2, column 1\n",
            file.display()
        )
    );
}

#[test]
fn lint_mismatched_html_close_errors_include_close_tag_location() {
    let dir = TestDir::new("lint_mismatched_html_close_location");
    let file = dir.write("input.html.erb", "<div>\n  <span>Hello</div>\n");

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: mismatched HTML close tag `</div>`, expected `</span>` at line 2, column 14\n",
            file.display()
        )
    );
}

#[test]
fn lint_unclosed_html_tag_errors_include_open_tag_location() {
    let dir = TestDir::new("lint_unclosed_html_location");
    let file = dir.write("input.html.erb", "<div>\n  <p>Hello</p>\n");

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: unclosed HTML tag `<div>` at line 1, column 1\n",
            file.display()
        )
    );
}

#[test]
fn lint_rule_diagnostics_include_line_and_column() {
    let dir = TestDir::new("lint_rule_location");
    let file = dir.write(
        "input.html.erb",
        "<p>Before</p>\n  <% while job.running? %>\n<p>Waiting</p>\n",
    );

    let output = run(["--lint".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "{}: unsupported ERB block starter `while` at line 2, column 3\n",
            file.display()
        )
    );
}

#[test]
fn multi_file_check_returns_failure_if_any_file_is_unformatted() {
    let dir = TestDir::new("multi_check");
    let formatted = dir.write("formatted.html.erb", FORMATTED);
    let unformatted = dir.write("unformatted.html.erb", UNFORMATTED);

    let output = run([
        "--check".as_ref(),
        formatted.as_path(),
        unformatted.as_path(),
    ]);

    assert_failure(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: file is formatted.\n", formatted.display())
    );
    assert_eq!(
        stderr(&output),
        format!("{}: file is not formatted.\n", unformatted.display())
    );
}

#[test]
fn multi_file_lint_returns_failure_if_any_file_has_diagnostics() {
    let dir = TestDir::new("multi_lint");
    let valid = dir.write("valid.html.erb", FORMATTED);
    let invalid = dir.write("invalid.html.erb", "<% if show_empty_state %>\n<% end %>\n");

    let output = run(["--lint".as_ref(), valid.as_path(), invalid.as_path()]);

    assert_failure(&output);
    assert_eq!(
        stdout(&output),
        format!("{}: no lint issues found.\n", valid.display())
    );
    assert_eq!(
        stderr(&output),
        format!(
            "{}: empty ERB control block `<% if show_empty_state %>` at line 1, column 1\n",
            invalid.display()
        )
    );
}

#[test]
fn multiple_files_without_mode_fails() {
    let dir = TestDir::new("multi_without_mode");
    let first = dir.write("first.html.erb", FORMATTED);
    let second = dir.write("second.html.erb", FORMATTED);

    let output = run([first.as_path(), second.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("multiple files require --write, --check, or --lint"));
}

#[test]
fn incompatible_mode_flags_fail() {
    let dir = TestDir::new("incompatible_flags");
    let file = dir.write("input.html.erb", FORMATTED);

    let output = run(["--write".as_ref(), "--check".as_ref(), file.as_path()]);

    assert_failure(&output);
    assert_eq!(stdout(&output), "");
    let stderr = stderr(&output);
    assert!(stderr.contains("--write"), "{stderr}");
    assert!(stderr.contains("--check"), "{stderr}");
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after unix epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("erbfmt-cli-{name}-{}-{unique}", std::process::id()));

        fs::create_dir_all(&path).unwrap();

        Self { path }
    }

    fn write(&self, name: &str, content: &str) -> PathBuf {
        let file = self.path.join(name);
        fs::write(&file, content).unwrap();
        file
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn run<I, P>(args: I) -> Output
where
    I: IntoIterator<Item = P>,
    P: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_erbfmt"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}
