class BearFormatter < Formula
  desc "Formatter for Bear notes"
  homepage "https://github.com/chasefinch/bear-formatter"
  url "https://github.com/chasefinch/bear-formatter/archive/refs/tags/v1.0.7.tar.gz"
  sha256 "9c3f31d598f46bb14b4e0dcf42cad2af8665682987717156cf9bb36f4a699a93"
  license "MIT"
  head "https://github.com/chasefinch/bear-formatter.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_equal "hello\n", shell_output("#{bin}/bear-format --code hello")
  end
end
