# Homebrew Formula for cc-gateway
# Install with: brew install --formula ./homebrew/Formula/cc-gateway.rb
# Or from tap: brew tap user/cc-gateway && brew install cc-gateway

class CcGateway < Formula
  desc "Pure Rust Claude API Gateway - OpenClaw alternative"
  homepage "https://github.com/user/cc-gateway"
  version "0.1.0"
  license "MIT"
  head "https://github.com/user/cc-gateway.git", branch: "main"

  # Dependencies for building from source
  depends_on "rust" => :build

  # Runtime dependencies
  depends_on "openssl@3"

  def install
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/cc-gateway"

    # Install shell completions
    generate_completions_from_executable(bin/"cc-gateway", "completions")

    # Install man page if available
    man1.install Dir["docs/*.1"] if Dir.exist?("docs")
  end

  test do
    # Test version flag
    assert_match "cc-gateway", shell_output("#{bin}/cc-gateway --version")

    # Test help flag
    assert_match "Usage", shell_output("#{bin}/cc-gateway --help")
  end
end
