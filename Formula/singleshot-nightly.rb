# typed: false
# frozen_string_literal: true

class SingleshotNightly < Formula
  desc "A CLI tool for testing AI models with a single prompt (nightly)"
  homepage "https://github.com/vincentzhangz/singleshot"
  version "0.1.0-nightly.20260204.99878b8"
  license "MIT"

  conflicts_with "singleshot", because: "both install a `singleshot` binary"

  on_macos do
    on_arm do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-aarch64-apple-darwin.tar.gz"
      sha256 "f23f84d70bbfd58bc4c9b0e5ddb9b0ae10ba85d2e8c849c835851e42621b2c3f"
    end
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-apple-darwin.tar.gz"
      sha256 "57a8a95e0c7d0c3f2b3aeffbc831d193ac5487a2a625ccdb234f589c74408f11"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0d7788d6c5316612ec8658b8e133c2f36f5bb37e75c928bb0f73c6a38c618389"
    end
  end

  def install
    bin.install "singleshot"
  end

  test do
    assert_match "singleshot", shell_output("#{bin}/singleshot --version")
  end
end
