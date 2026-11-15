use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "erbfmt-json-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, text: &str) {
        fs::write(self.0.join(name), text).unwrap();
    }
    fn run(&self, args: &[&str], source: Option<&str>) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_erbfmt"))
            .args(args)
            .current_dir(&self.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(source) = source {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(source.as_bytes())
                .unwrap();
        }
        child.wait_with_output().unwrap()
    }
    fn lint(&self, source: &str) -> Value {
        let output = self.run(
            &[
                "--lint",
                "--lint-format",
                "json",
                "--stdin-filepath",
                "input.html.erb",
            ],
            Some(source),
        );
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if output.stdout.is_empty() {
            return Value::Null;
        }
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn assist(&self, source: &str) -> Value {
        let output = self.run(
            &[
                "--assist",
                "--assist-format",
                "json",
                "--stdin-filepath",
                "input.html.erb",
            ],
            Some(source),
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn apply(source: &str, edit: &Value) -> String {
    let start = edit["range"]["start"].as_u64().unwrap() as usize;
    let end = edit["range"]["end"].as_u64().unwrap() as usize;
    format!(
        "{}{}{}",
        &source[..start],
        edit["replacement"].as_str().unwrap(),
        &source[end..]
    )
}

#[test]
fn json_streams_only_lint_findings_and_has_stable_summaries() {
    let fixture = Fixture::new();
    fixture.write(
        "erbfmt.json",
        r#"{"linter":{"rules":{"noDeprecatedHtmlTag":"warn"}}}"#,
    );
    fixture.write("clean.html.erb", "<p>Clean</p>\n");
    fixture.write("one.html.erb", "<center>One</center>\n");
    fixture.write("two.html.erb", "<center>Two</center>\n");
    let output = fixture.run(
        &[
            "--lint",
            "--lint-format",
            "json",
            "clean.html.erb",
            "one.html.erb",
            "two.html.erb",
        ],
        None,
    );
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    for record in records {
        assert!(record["file"].as_str().unwrap().ends_with(".html.erb"));
        let findings = record["diagnostics"].as_array().unwrap();
        assert!(!findings.is_empty());
        assert_eq!(record["summary"]["issues"], findings.len());
        assert_eq!(record["summary"]["errors"], 0);
        assert_eq!(record["summary"]["warnings"], findings.len());
        assert_eq!(record["summary"]["infos"], 0);
        for diagnostic in findings {
            assert_eq!(diagnostic["severity"], "warning");
            assert_eq!(diagnostic["rule"], "noDeprecatedHtmlTag");
            assert_eq!(diagnostic["location"]["line"], 1);
        }
    }
}

#[test]
fn unicode_locations_count_scalars_but_assist_ranges_count_utf8_bytes() {
    let fixture = Fixture::new();
    let source = "<p>日本語😀</p>\r\n<input title='名前😀'>\r\n";
    let lint = fixture.lint(source);
    let diagnostic = &lint["diagnostics"][0];
    assert_eq!(diagnostic["rule"], "noNonDoubleQuotedHtmlAttributeValue");
    assert_eq!(
        diagnostic["location"],
        serde_json::json!({"line":2,"column":8})
    );
    let assist = fixture.assist(source);
    let action = &assist["actions"][0];
    assert_eq!(action["id"], "html.useDoubleQuotedAttributes");
    assert_eq!(action["rule"], diagnostic["rule"]);
    let start = action["range"]["start"].as_u64().unwrap() as usize;
    let end = action["range"]["end"].as_u64().unwrap() as usize;
    assert_eq!(&source[start..end], "title='名前😀'");
    assert_eq!(action["location"], serde_json::json!({"line":2,"column":8}));
    assert_eq!(
        apply(source, &action["edit"]),
        source.replace("'名前😀'", "\"名前😀\"")
    );
    let ignored = apply(source, &diagnostic["ignore"]);
    assert!(ignored.contains("%>\r\n<input"));
    assert!(fixture.lint(&ignored).is_null());
}

#[test]
fn syntax_diagnostics_have_structured_locations_and_cannot_be_ignored() {
    let fixture = Fixture::new();
    for (source, line, column) in [
        ("<p>日本語😀</p>\n<div>\n", 2, 1),
        ("<p>日本語😀</p><%= value", 1, 12),
    ] {
        let lint = fixture.lint(source);
        let diagnostic = &lint["diagnostics"][0];
        assert!(diagnostic["rule"].is_null());
        assert!(diagnostic["ignore"].is_null());
        assert_eq!(diagnostic["severity"], "error");
        assert_eq!(
            diagnostic["location"],
            serde_json::json!({"line":line,"column":column})
        );
        assert!(
            !diagnostic["message"]
                .as_str()
                .unwrap()
                .contains(" at line ")
        );
    }
}

#[test]
fn config_and_io_failures_are_stderr_not_lint_json() {
    let fixture = Fixture::new();
    fixture.write("bad.json", "{invalid");
    for args in [
        vec!["--lint", "--lint-format", "json", "missing.html.erb"],
        vec![
            "--config",
            "bad.json",
            "--lint",
            "--lint-format",
            "json",
            "input.html.erb",
        ],
    ] {
        let output = fixture.run(&args, None);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    let output = fixture.run(&["--lint-format", "invalid"], None);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn exclusions_skip_json_and_zero_assists_still_return_a_record() {
    let fixture = Fixture::new();
    let assist = fixture.assist("<p>Clean</p>\n");
    assert_eq!(assist["summary"]["actions"], 0);
    assert_eq!(assist["actions"], serde_json::json!([]));
    assert!(!assist["ignoreTargets"].as_array().unwrap().is_empty());
    assert!(fixture.lint("<p>Clean</p>\n").is_null());
    fixture.write(
        "erbfmt.json",
        r#"{"files":{"includes":["app/views/**/*.html.erb"]}}"#,
    );
    let output = fixture.run(
        &[
            "--assist",
            "--assist-format",
            "json",
            "--stdin-filepath",
            "ignored.html.erb",
        ],
        Some("<% %>"),
    );
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn ignore_boundaries_are_real_nodes_and_consecutive_selectors_compose() {
    let fixture = Fixture::new();
    let source = "<%# erbfmt-ignore lint/noDeprecatedHtmlTag: legacy %>\n<center title='name'>Legacy</center>\n";
    let lint = fixture.lint(source);
    let diagnostic = &lint["diagnostics"][0];
    assert_eq!(diagnostic["rule"], "noNonDoubleQuotedHtmlAttributeValue");
    let ignored = apply(source, &diagnostic["ignore"]);
    assert!(fixture.lint(&ignored).is_null());
    let assist = fixture.assist(&ignored);
    // Explicit assist remains available even when its lint is suppressed.
    assert_eq!(assist["summary"]["actions"], 1);
    let target = &assist["ignoreTargets"][0];
    let formatted_ignore = apply(&ignored, &target["format"]);
    let output = fixture.run(
        &["--stdin-filepath", "input.html.erb"],
        Some(&formatted_ignore),
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), formatted_ignore);
    for source in [
        "<input\n title='name'\n>\n",
        "<script>\nconst html = '<center>Text</center>';\n</script>\n",
        "<% value = \"\n<center>Text</center>\n\" %>\n",
    ] {
        let assist = fixture.assist(source);
        assert!(
            assist["ignoreTargets"]
                .as_array()
                .unwrap()
                .iter()
                .all(|target| target["line"] != 2)
        );
        if source.starts_with("<input") {
            assert!(fixture.lint(source)["diagnostics"][0]["ignore"].is_null());
        }
    }
}

#[test]
fn fragment_tolerance_does_not_claim_to_balance_cross_erb_html() {
    let fixture = Fixture::new();
    // Reduced from the audit's rejected cross-branch HTML. Do not repair this
    // by inventing closing tags, including when fragment tolerance is enabled.
    let source = "<% if visible %>\n<div>\n<% else %>\n</div>\n<% end %>\n";
    for relaxed in [false, true] {
        fixture.write(
            "erbfmt.json",
            &format!(r#"{{"parser":{{"allowUnbalancedHtmlFragments":{relaxed}}}}}"#),
        );
        let output = fixture.run(&["--stdin-filepath", "input.html.erb"], Some(source));
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn json_severity_changes_exit_status_and_keeps_the_rule_id() {
    let fixture = Fixture::new();
    for (severity, label, success, count) in [
        ("info", "info", true, "infos"),
        ("warn", "warning", true, "warnings"),
        ("error", "error", false, "errors"),
    ] {
        fixture.write(
            "erbfmt.json",
            &format!(r#"{{"linter":{{"rules":{{"noDeprecatedHtmlTag":"{severity}"}}}}}}"#),
        );
        let output = fixture.run(
            &[
                "--lint",
                "--lint-format",
                "json",
                "--stdin-filepath",
                "input.html.erb",
            ],
            Some("<center>Legacy</center>\n"),
        );
        assert_eq!(output.status.success(), success);
        assert!(output.stderr.is_empty());
        let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
        let findings = payload["diagnostics"].as_array().unwrap();
        assert_eq!(payload["summary"][count], findings.len());
        for finding in findings {
            assert_eq!(finding["rule"], "noDeprecatedHtmlTag");
            assert_eq!(finding["severity"], label);
        }
    }
}
