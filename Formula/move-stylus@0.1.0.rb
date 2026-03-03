class MoveStylusAT010 < Formula
  desc "Move compiler for Arbitrum's Stylus"
  homepage "https://move-stylus.ratherlabs.com/"
  version "0.1.0"
  license "BUSL-1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-aarch64-macos-#{version}.tar.gz"
      sha256 "0e9a7ce52f37fb9c4281af6eb6bbefdc6e2104b0294f8cc756477470cfbf9c2b"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-aarch64-linux-#{version}.tar.gz"
      sha256 "05d00bd4de1b639fecfaae8616eb3d0cee0d916cd7030bfd6b0d5f96d5dbe246"
    elsif Hardware::CPU.intel?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-x86_64-linux-#{version}.tar.gz"
      sha256 "4980c8bae72e59ca27fb9768e79666dff4f577f43052e0ffa4a9c0fdae5dad83"
    end
  end

  def install
    bin.install "move-stylus"
  end

  test do
    system "#{bin}/move-stylus", "--version"
  end
end
