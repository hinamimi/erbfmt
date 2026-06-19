# frozen_string_literal: true

require_relative "lib/erbfmt/version"

Gem::Specification.new do |spec|
  spec.name = "erbfmt"
  spec.version = Erbfmt::VERSION
  spec.authors = ["erbfmt contributors"]
  spec.summary = "Ruby wrapper for the erbfmt formatter and linter"
  spec.description = "A thin Ruby launcher for the platform-specific erbfmt Rust binary."
  spec.homepage = "https://github.com/hinamimi/erbfmt"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.1"
  spec.metadata = {
    "source_code_uri" => "https://github.com/hinamimi/erbfmt",
    "bug_tracker_uri" => "https://github.com/hinamimi/erbfmt/issues",
    "changelog_uri" => "https://github.com/hinamimi/erbfmt/releases"
  }

  spec.bindir = "exe"
  spec.executables = ["erbfmt"]
  spec.require_paths = ["lib"]
  spec.platform = Gem::Platform.new(
    ENV.fetch("ERBFMT_GEM_PLATFORM", Gem::Platform.local.to_s)
  )
  spec.files = [
    "LICENSE.txt",
    "README.md",
    "exe/erbfmt",
    "lib/erbfmt.rb",
    "lib/erbfmt/binary.rb",
    "lib/erbfmt/version.rb"
  ] + Dir["libexec/erbfmt-bin*"]

  spec.add_development_dependency "minitest", "~> 5.25"
  spec.add_development_dependency "rake", "~> 13.2"
end
