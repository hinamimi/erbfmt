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
            "{}: empty ERB control block `<% if show_empty_state %>`\n",
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
            "{}: empty ERB control block `<% if show_empty_state %>`\n",
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
