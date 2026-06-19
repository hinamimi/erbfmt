# frozen_string_literal: true

require_relative "test_helper"

class BinaryTest < Minitest::Test
  def test_resolves_an_executable_override
    Dir.mktmpdir("erbfmt-binary") do |directory|
      binary = File.join(directory, "erbfmt")
      File.write(binary, "#!/bin/sh\nexit 0\n")
      File.chmod(0o755, binary)

      assert_equal binary, Erbfmt::Binary.path("ERBFMT_BINARY" => binary)
    end
  end

  def test_rejects_a_missing_override
    error = assert_raises(Erbfmt::BinaryNotFound) do
      Erbfmt::Binary.path("ERBFMT_BINARY" => "/missing/erbfmt")
    end

    assert_includes error.message, "ERBFMT_BINARY is missing or not executable"
  end

  def test_reports_a_missing_packaged_binary
    error = assert_raises(Erbfmt::BinaryNotFound) do
      Erbfmt::Binary.path({})
    end

    assert_includes error.message, "packaged erbfmt binary is missing or not executable"
  end
end
