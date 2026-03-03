class MoveStylusAT010-test < Formula
  desc "Move compiler for Arbitrum's Stylus"
  homepage "https://move-stylus.ratherlabs.com/"
  version "0.1.0-test"
  license "BUSL-1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-aarch64-macos-#{version}.tar.gz"
      sha256 "cb2d93a85fd798547efd640c6a9e65061088b002920427abfda37f2b31e24170"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-aarch64-linux-#{version}.tar.gz"
      sha256 "676468423bc26cf96758a83f57acf2b3fda6662e8ed2b0d91eae507670c6a3da"
    elsif Hardware::CPU.intel?
      url "https://github.com/rather-labs/move-stylus/releases/download/v#{version}/move-stylus-x86_64-linux-#{version}.tar.gz"
      sha256 "af0e7a2f427cbe6eede0fc8243ecff113466a01949335b40b9596a84e58da300"
    end
  end

  def install
    bin.install "move-stylus"
  end

  test do
    system "#{bin}/move-stylus", "--version"
  end
end
