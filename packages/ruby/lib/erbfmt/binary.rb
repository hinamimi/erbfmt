# frozen_string_literal: true

module Erbfmt
  class BinaryNotFound < StandardError; end

  module Binary
    module_function

    def path(env = ENV)
      override = env["ERBFMT_BINARY"]
      return validate(File.expand_path(override), "ERBFMT_BINARY") unless override.nil? || override.empty?

      validate(packaged_path, "packaged erbfmt binary")
    end

    def packaged_path
      name = Gem.win_platform? ? "erbfmt-bin.exe" : "erbfmt-bin"
      File.expand_path("../../libexec/#{name}", __dir__)
    end

    def validate(candidate, source)
      return candidate if File.file?(candidate) && File.executable?(candidate)

      raise BinaryNotFound,
        "#{source} is missing or not executable: #{candidate}"
    end
    private_class_method :validate
  end
end
