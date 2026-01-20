# Homebrew formula for OxideKit CLI
# Install: brew install oxidekit/tap/oxide

class Oxide < Formula
  desc "Rust-native application platform CLI"
  homepage "https://oxidekit.com"
  version "0.3.0"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/oxidekit/oxidekit-core/releases/download/v#{version}/oxide-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/oxidekit/oxidekit-core/releases/download/v#{version}/oxide-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X64"
    end
  end

  on_linux do
    url "https://github.com/oxidekit/oxidekit-core/releases/download/v#{version}/oxide-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "oxide"
  end

  test do
    assert_match "oxide #{version}", shell_output("#{bin}/oxide --version")
  end
end
