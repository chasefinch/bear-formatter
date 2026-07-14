class BearFormatter < Formula
  desc "Cute little formatter for Bear notes"
  homepage "https://github.com/chasefinch/bear-formatter"
  url "https://github.com/chasefinch/bear-formatter/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "6c3acdbfbacd06b3fa3c3b0b961615f71e6cad140526061eb439566c1ca35f22"
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
