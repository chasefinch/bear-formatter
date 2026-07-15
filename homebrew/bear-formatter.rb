class BearFormatter < Formula
  desc "Formatter for Bear notes"
  homepage "https://github.com/chasefinch/bear-formatter"
  url "https://github.com/chasefinch/bear-formatter/archive/refs/tags/v1.0.3.tar.gz"
  sha256 "4749563a06dc5d328fee67334f99885c29cc7a722e82f9131e299de1ed94052a"
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
