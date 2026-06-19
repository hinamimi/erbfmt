# frozen_string_literal: true

require_relative "test_helper"

class IntegrationTest < Minitest::Test
  def setup
    skip "build erbfmt with cargo build before running integration tests" unless File.executable?(RUST_BINARY)
  end

  def test_launcher_forwards_arguments_to_the_rust_binary
    Dir.mktmpdir("erbfmt-integration") do |directory|
      template = File.join(directory, "input.html.erb")
      File.write(template, "<div>\n<p>Hello</p>\n</div>\n")

      stdout, stderr, status = run_launcher(template)

      assert status.success?, stderr
      assert_equal "<div>\n  <p>Hello</p>\n</div>\n", stdout
      assert_empty stderr
    end
  end

  def test_launcher_preserves_the_rust_exit_status
    stdout, _stderr, status = run_launcher("--check", "/missing/input.html.erb")

    refute status.success?
    assert_empty stdout
  end

  def test_launcher_reports_the_rust_version
    stdout, stderr, status = run_launcher("--version")

    assert status.success?, stderr
    assert_match(/^erbfmt \S+\n$/, stdout)
    assert_empty stderr
  end

  private

  def run_launcher(*arguments)
    Open3.capture3(
      { "ERBFMT_BINARY" => RUST_BINARY },
      Gem.ruby,
      "-I#{File.join(PACKAGE_ROOT, "lib")}",
      File.join(PACKAGE_ROOT, "exe", "erbfmt"),
      *arguments
    )
  end
end
