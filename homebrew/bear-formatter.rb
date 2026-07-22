class BearFormatter < Formula
  desc "Formatter for Bear notes"
  homepage "https://github.com/chasefinch/bear-formatter"
  url "https://github.com/chasefinch/bear-formatter/archive/refs/tags/v1.0.5.tar.gz"
  sha256 "dbeb8532aa4f6d3319943228a4203991a6e3fc5285d53bef1fd4e168757886ed"
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
