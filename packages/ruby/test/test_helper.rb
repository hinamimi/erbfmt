# frozen_string_literal: true

require "minitest/autorun"
require "open3"
require "tmpdir"
require "erbfmt"

PACKAGE_ROOT = File.expand_path("..", __dir__)
REPOSITORY_ROOT = File.expand_path("../..", PACKAGE_ROOT)
RUST_BINARY = File.expand_path(
  ENV.fetch("ERBFMT_BINARY", "target/debug/erbfmt"),
  REPOSITORY_ROOT
)
