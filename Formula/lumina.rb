class Lumina < Formula
  desc "Declarative reactive language for IoT and infrastructure monitoring"
  homepage "https://lumina-lang.dev"
  version "1.7.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/IshimweIsaac/Lumina/releases/download/v1.7.0/lumina-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_ARM64" # To be updated during Phase 3 verification or manual release
    else
      url "https://github.com/IshimweIsaac/Lumina/releases/download/v1.7.0/lumina-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_X64"
    end
  end

  def install
    # On macOS, the tarball contains 'lumina-macos-arm64' or 'lumina-macos-x64'
    # and their respective LSP binaries.
    
    # Identify the binary name based on architecture
    bin_name = Hardware::CPU.arm? ? "lumina-macos-arm64" : "lumina-macos-x64"
    
    bin.install bin_name => "lumina"
    bin.install "#{bin_name}-lsp" => "lumina-lsp"
  end

  test do
    system "#{bin}/lumina", "--version"
  end
end
