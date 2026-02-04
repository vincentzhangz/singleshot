class SingleshotNightly < Formula
  desc "A CLI tool for testing AI models with a single prompt (nightly)"
  homepage "https://github.com/vincentzhangz/singleshot"
  version "0.2.0-nightly.20260204.59eae35"
  license "MIT"

  conflicts_with "singleshot", because: "both install a singleshot binary"

  on_macos do
    on_arm do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-aarch64-apple-darwin.tar.gz"
      sha256 "edba981ea40f68d957382457465f2c7ecba6d567a9560a1cdf378c002931be67"
    end
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-apple-darwin.tar.gz"
      sha256 "b1752222c1aae3afec3e4507b746144cf7a2c74c2fbf892d7c49cdb2e0fe82ef"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "1bad7d2d88a06065b163032c4a1b7a5df636969cc58dc4e295f92b52b9c0456d"
    end
  end

  def install
    bin.install "singleshot"
  end

  test do
    assert_match "singleshot", shell_output("#{bin}/singleshot --version")
  end
end
