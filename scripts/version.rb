#!/usr/bin/env ruby
# frozen_string_literal: true

require "json"
require "rubygems/version"

module ErbfmtVersioning
  class Error < StandardError; end

  ROOT = File.expand_path("..", __dir__)
  VERSION_PATTERN = /\A(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?\z/
  VSCODE_VERSION_DOCS = [
    "docs/VSCode.md",
    "editors/vscode/README.md",
    "editors/vscode/README_ja.md"
  ].freeze

  module_function

  def set(version, root = ROOT)
    validate_version(version)
    current = cargo_toml_version(root)
    verify(root, expected: current)
    gem_version = ruby_gem_version(version)

    replace_once(
      path(root, "Cargo.toml"),
      /^version = "#{Regexp.escape(current)}"$/,
      "version = \"#{version}\""
    )
    replace_once(
      path(root, "Cargo.lock"),
      /(\[\[package\]\]\nname = "erbfmt"\nversion = ")[^"]+("\n)/,
      "\\1#{version}\\2"
    )
    replace_once(
      path(root, "packages/ruby/lib/erbfmt/version.rb"),
      /^  VERSION = "[^"]+"$/,
      "  VERSION = \"#{gem_version}\""
    )
    replace_once(
      path(root, "packages/ruby/Gemfile.lock"),
      /^    erbfmt \([^)]+\)$/,
      "    erbfmt (#{gem_version})"
    )

    update_json_version(path(root, "editors/vscode/package.json"), version)
    update_package_lock(path(root, "editors/vscode/package-lock.json"), version)
    VSCODE_VERSION_DOCS.each do |relative|
      replace_all(
        path(root, relative),
        /erbfmt-vscode-#{Regexp.escape(current)}\.vsix/,
        "erbfmt-vscode-#{version}.vsix"
      )
    end

    verify(root, expected: version)
  end

  def verify(root = ROOT, expected: nil)
    versions = collect_versions(root)
    expected ||= versions.fetch(:cargo_toml)
    expected_gem = ruby_gem_version(expected)
    mismatches = []

    %i[cargo_toml cargo_lock vscode_package vscode_lock vscode_lock_root].each do |key|
      mismatches << "#{key}=#{versions.fetch(key)}" unless versions.fetch(key) == expected
    end
    %i[ruby_gem ruby_gem_lock].each do |key|
      mismatches << "#{key}=#{versions.fetch(key)}" unless versions.fetch(key) == expected_gem
    end
    VSCODE_VERSION_DOCS.each do |relative|
      values = File.read(path(root, relative)).scan(/erbfmt-vscode-([0-9A-Za-z.-]+)\.vsix/).flatten.uniq
      mismatches << "#{relative}=#{values.join(",")}" unless values == [expected]
    end

    return versions if mismatches.empty?

    raise Error, "version mismatch (expected #{expected}): #{mismatches.join(", ")}"
  end

  def collect_versions(root = ROOT)
    vscode_package = read_json(path(root, "editors/vscode/package.json"))
    vscode_lock = read_json(path(root, "editors/vscode/package-lock.json"))

    {
      cargo_toml: cargo_toml_version(root),
      cargo_lock: capture_once(
        path(root, "Cargo.lock"),
        /\[\[package\]\]\nname = "erbfmt"\nversion = "([^"]+)"\n/
      ),
      ruby_gem: capture_once(
        path(root, "packages/ruby/lib/erbfmt/version.rb"),
        /^  VERSION = "([^"]+)"$/
      ),
      ruby_gem_lock: capture_once(
        path(root, "packages/ruby/Gemfile.lock"),
        /^    erbfmt \(([^)]+)\)$/
      ),
      vscode_package: vscode_package.fetch("version"),
      vscode_lock: vscode_lock.fetch("version"),
      vscode_lock_root: vscode_lock.fetch("packages").fetch("").fetch("version")
    }
  end

  def cargo_toml_version(root)
    capture_once(path(root, "Cargo.toml"), /^version = "([^"]+)"$/)
  end

  def ruby_gem_version(version)
    version.tr("-", ".")
  end

  def validate_version(version)
    return if VERSION_PATTERN.match?(version) && Gem::Version.correct?(ruby_gem_version(version))

    raise Error, "invalid release version: #{version.inspect}"
  end

  def replace_once(file, pattern, replacement)
    content = File.read(file)
    matches = content.scan(pattern).length
    raise Error, "expected one version entry in #{file}, found #{matches}" unless matches == 1

    File.write(file, content.sub(pattern, replacement))
  end

  def capture_once(file, pattern)
    matches = File.read(file).scan(pattern)
    raise Error, "expected one version entry in #{file}, found #{matches.length}" unless matches.length == 1

    value = matches.first
    value.is_a?(Array) ? value.first : value
  end

  def replace_all(file, pattern, replacement)
    content = File.read(file)
    matches = content.scan(pattern).length
    raise Error, "expected at least one version entry in #{file}" if matches.zero?

    File.write(file, content.gsub(pattern, replacement))
  end

  def update_json_version(file, version)
    document = read_json(file)
    document["version"] = version
    write_json(file, document)
  end

  def update_package_lock(file, version)
    document = read_json(file)
    document["version"] = version
    document.fetch("packages").fetch("")["version"] = version
    write_json(file, document)
  end

  def read_json(file)
    JSON.parse(File.read(file))
  rescue JSON::ParserError => error
    raise Error, "invalid JSON in #{file}: #{error.message}"
  end

  def write_json(file, document)
    File.write(file, "#{JSON.pretty_generate(document)}\n")
  end

  def path(root, relative)
    File.join(root, relative)
  end
end

if $PROGRAM_NAME == __FILE__
  command = ARGV.shift
  root = ENV.fetch("ERBFMT_VERSION_ROOT", ErbfmtVersioning::ROOT)

  begin
    case command
    when "set"
      version = ARGV.shift
      raise ErbfmtVersioning::Error, "usage: scripts/version.rb set VERSION" if version.nil? || !ARGV.empty?

      ErbfmtVersioning.set(version, root)
      puts "set erbfmt version to #{version}"
    when "verify"
      expected = ARGV.shift
      raise ErbfmtVersioning::Error, "usage: scripts/version.rb verify [VERSION]" unless ARGV.empty?

      versions = ErbfmtVersioning.verify(root, expected: expected)
      puts "verified erbfmt version #{versions.fetch(:cargo_toml)}"
    else
      raise ErbfmtVersioning::Error, "usage: scripts/version.rb <set VERSION|verify [VERSION]>"
    end
  rescue ErbfmtVersioning::Error, KeyError => error
    warn error.message
    exit 1
  end
end
