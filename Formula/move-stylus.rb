class MoveStylus < Formula
  desc "Move compiler for Arbitrum's Stylus"
  homepage "https://move-stylus.ratherlabs.com/"
  version "0.1.0"
  license "BUSL-1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-aarch64-macos-#{version}.tar.gz"
      sha256 "3a9017795e13fdb07e17b7f461f4e84cf852966468a4d5856c83b4c84a896947"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-aarch64-linux-#{version}.tar.gz"
      sha256 "1b87e6b194672983841356ccbf2f6c10117f115574366cc10956f9ff506f303e"
    elsif Hardware::CPU.intel?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-x86_64-linux-#{version}.tar.gz"
      sha256 "e651ca1dbfd9665287a9e53c86f8787e180a6fbd61b757bb397d68db24415774"
    end
  end

  def install
    bin.install "move-stylus"
  end

  test do
    system "#{bin}/move-stylus", "--version"
  end
end
