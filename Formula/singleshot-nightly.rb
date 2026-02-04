class SingleshotNightly < Formula
  desc "A CLI tool for testing AI models with a single prompt (nightly)"
  homepage "https://github.com/vincentzhangz/singleshot"
  version "0.1.0-nightly.20260204.9be020a"
  license "MIT"

  conflicts_with "singleshot", because: "both install a singleshot binary"

  on_macos do
    on_arm do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-aarch64-apple-darwin.tar.gz"
      sha256 "b677ea8414fce499790ee1eb693ef33f940734ae977303d0e3c41cd7e45923cf"
    end
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-apple-darwin.tar.gz"
      sha256 "44504edbbf74efcfeff6946d90a3c5352951f566ff1782f527f76410da58ee0c"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vincentzhangz/singleshot/releases/download/nightly/singleshot-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "a783b2ede98b9ef9f84702bc2edbb83f16d976d48fa8fb435e4deb94a279030b"
    end
  end

  def install
    bin.install "singleshot"
  end

  test do
    assert_match "singleshot", shell_output("#{bin}/singleshot --version")
  end
end
