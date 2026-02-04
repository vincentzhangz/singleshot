class SingleshotNightly < Formula
  desc "A CLI tool for testing AI models with a single prompt (nightly)"
  homepage "https://github.com/vincentzhangz/singleshot"
  version "0.2.0-nightly.20260204.1c9a5d7"
  license "MIT"

  conflicts_with "singleshot", because: "both install a singleshot binary"

  on_macos do
    on_arm do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-aarch64-apple-darwin.tar.gz"
      sha256 "6ba3c8c8d761fc20c26a7e166be6928b2dd0a6716936d4ff967c2c8728189ff0"
    end
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-apple-darwin.tar.gz"
      sha256 "072268546399d245510d33e9d91b43d6d7073c2000d1df47ba908b01b789dd5a"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "9fdb206db8418fd651a19c820c9db1a7960115dca7bc61440f8a5018bb15ab8a"
    end
  end

  def install
    bin.install "singleshot"
  end

  test do
    assert_match "singleshot", shell_output("#{bin}/singleshot --version")
  end
end
