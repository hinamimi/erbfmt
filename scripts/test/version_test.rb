# frozen_string_literal: true

require "minitest/autorun"
require "fileutils"
require "tmpdir"
require_relative "../version"

class VersionTest < Minitest::Test
  VERSION_FILES = [
    "Cargo.toml",
    "Cargo.lock",
    "packages/ruby/lib/erbfmt/version.rb",
    "packages/ruby/Gemfile.lock",
    "editors/vscode/package.json",
    "editors/vscode/package-lock.json",
    *ErbfmtVersioning::VSCODE_VERSION_DOCS
  ].freeze

  def test_sets_and_verifies_a_stable_version_in_an_isolated_copy
    repository_version = ErbfmtVersioning.collect_versions.fetch(:cargo_toml)

    with_version_files do |root|
      versions = ErbfmtVersioning.set("1.2.3", root)

      assert_equal "1.2.3", versions.fetch(:cargo_toml)
      assert_equal "1.2.3", versions.fetch(:ruby_gem)
      assert_equal "1.2.3", versions.fetch(:vscode_lock_root)
      assert_equal repository_version, ErbfmtVersioning.collect_versions.fetch(:cargo_toml)
    end
  end

  def test_reports_a_version_mismatch
    with_version_files do |root|
      package = File.join(root, "editors/vscode/package.json")
      document = JSON.parse(File.read(package))
      document["version"] = "9.9.9"
      File.write(package, "#{JSON.pretty_generate(document)}\n")

      error = assert_raises(ErbfmtVersioning::Error) do
        ErbfmtVersioning.verify(root)
      end

      assert_includes error.message, "vscode_package=9.9.9"
    end
  end

  def test_rejects_an_invalid_version
    error = assert_raises(ErbfmtVersioning::Error) do
      ErbfmtVersioning.validate_version("release-1")
    end

    assert_includes error.message, "invalid release version"
  end

  private

  def with_version_files
    Dir.mktmpdir("erbfmt-version") do |root|
      VERSION_FILES.each do |relative|
        source = File.join(ErbfmtVersioning::ROOT, relative)
        destination = File.join(root, relative)
        FileUtils.mkdir_p(File.dirname(destination))
        FileUtils.cp(source, destination)
      end

      yield root
    end
  end
end
