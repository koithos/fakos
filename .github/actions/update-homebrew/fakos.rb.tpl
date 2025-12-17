# Formula/fakos.rb
class fakos < Formula
  desc "A CLI tool for fakos"
  homepage "https://github.com/koithos/fakos"
  version "${version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "${amd64_url}"
      sha256 "${amd64_sha256}"
    end
    if Hardware::CPU.arm?
      url "${arm64_url}"
      sha256 "${arm64_sha256}"
    end
  end

  def install
    if Hardware::CPU.intel?
      bin.install "fakos-x86_64-apple-darwin" => "fakos"
    elsif Hardware::CPU.arm?
      bin.install "fakos-aarch64-apple-darwin" => "fakos"
    end
  end

  test do
    system "#{bin}/fakos", "--version"
  end
end
