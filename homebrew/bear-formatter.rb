class BearFormatter < Formula
  desc "Formatter for Bear notes"
  homepage "https://github.com/chasefinch/bear-formatter"
  url "https://github.com/chasefinch/bear-formatter/archive/refs/tags/v1.0.1.tar.gz"
  sha256 "6e7b4f1733c2b5e34769780876ba521bee1ccea5357bb6451fda1bb2fe2cb4e4"
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
