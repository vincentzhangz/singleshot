# typed: false
# frozen_string_literal: true

class SingleshotNightly < Formula
  desc "A CLI tool for testing AI models with a single prompt (nightly)"
  homepage "https://github.com/vincentzhangz/singleshot"
  version "0.1.0-nightly.20260204.d36e963"
  license "MIT"

  conflicts_with "singleshot", because: "both install a `singleshot` binary"

  on_macos do
    on_arm do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-aarch64-apple-darwin.tar.gz"
      sha256 "1779f687cdd45e5efd82cdd211ddbaad93197d114940f34e23d2953acf6508a0"
    end
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-apple-darwin.tar.gz"
      sha256 "eaf183581ff1679d5fc173657c22804c3015df59c78132de0302178e068afc73"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "4ca25628aa96d127ec7d68ed60c4008323a4fba67b1be5ea62bbd4cba28d8166"
    end
  end

  def install
    bin.install "singleshot"
  end

  test do
    assert_match "singleshot", shell_output("#{bin}/singleshot --version")
  end
end
