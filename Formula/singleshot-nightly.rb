# typed: false
# frozen_string_literal: true

class SingleshotNightly < Formula
  desc "A CLI tool for testing AI models with a single prompt (nightly)"
  homepage "https://github.com/vincentzhangz/singleshot"
  version "0.1.0-nightly.20260204.732a6e3"
  license "MIT"

  conflicts_with "singleshot", because: "both install a `singleshot` binary"

  on_macos do
    on_arm do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-aarch64-apple-darwin.tar.gz"
      sha256 "b7cf7d830457358a82b2f92e7eb1bbcb167447e4223bc3b55609518a6641f843"
    end
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-apple-darwin.tar.gz"
      sha256 "b5093ddf6f1a406937e33389ad9ca40bb6a433b01247041cb3f4cd75416c6e30"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "458acc2fcca4109bbfc4c72ddb1ff00e03cc237c17fccf19451d84b9fca1307c"
    end
  end

  def install
    bin.install "singleshot"
  end

  test do
    assert_match "singleshot", shell_output("#{bin}/singleshot --version")
  end
end
