# typed: false
# frozen_string_literal: true

class Singleshot < Formula
  desc "A CLI tool for testing AI models with a single prompt"
  homepage "https://github.com/vincentzhangz/singleshot"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/vincentzhangz/singleshot/releases/download/v#{version}/singleshot-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM"
    end
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/v#{version}/singleshot-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X86"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/v#{version}/singleshot-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX"
    end
  end

  def install
    bin.install "singleshot"
  end

  test do
    assert_match "singleshot", shell_output("#{bin}/singleshot --version")
  end
end
