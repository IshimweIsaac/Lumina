**LUMINA**

**v1.7 Implementation Guide**

**The Experience Release -- Build Guide**

_Exact specifications for website, installer, package managers, marketplace, guides, Docker, and playground._

_"Describe what is true. Lumina figures out what to do."_

Chapters 41-48 | No new language features | Distribution and experience | 2026

_Designed and authored by Isaac Ishimwe_

**Preface**

**How To Use This Document**

_This is a build guide -- not a language guide. No Rust. No crates. Pure distribution._

**CRITICAL Critical -- Read Before Building Anything**

v1.7 adds ZERO new language features.

Do NOT touch any Rust crate during v1.7.

Do NOT modify lumina-parser, lumina-runtime, lumina-analyzer, or any other crate.

v1.6 must be fully complete and cargo test --workspace green before v1.7 begins.

v1.7 is entirely: website, scripts, config files, documentation, and frontend.

The only code written in v1.7 is: shell scripts, HTML/CSS/JS, YAML, and Markdown.

**NOTE What v1.7 Requires from v1.6**

lumina binary -- compiled, tested, all features working

lumina-lsp binary -- LSP v2 complete with rename/references/code actions

lumina-wasm -- WASM binary built with wasm-pack, playground v2 working

All error codes L001-L042 implemented in the analyzer

cargo test --workspace 100% green

No known runtime bugs

| **Chapter** | **What gets built**                                |
| ----------- | -------------------------------------------------- |
| 41          | Static website -- lumina-lang.dev                  |
| 42          | install.sh -- the one-line installer script        |
| 43          | Homebrew Formula + GitHub releases binary pipeline |
| 44          | VS Code marketplace publication                    |
| 45          | Four getting started guides in Markdown            |
| 46          | Error message rewrites L001-L042                   |
| 47          | Dockerfile + Docker Hub publication                |
| 48          | Playground performance optimizations + polish      |

**Chapter 41**

**The Website**

_lumina-lang.dev -- static site, playground embedded, fast, alive on arrival_

# **41.1 Tech Stack Decision**

| **Choice**            | **Specification**                                                        |
| --------------------- | ------------------------------------------------------------------------ |
| Static site generator | Astro or plain HTML/CSS -- no React, no Next.js, no server               |
| Hosting               | Cloudflare Pages or Netlify -- free tier, CDN globally, deploys from git |
| Domain                | lumina-lang.dev -- register via Cloudflare or Namecheap                  |
| Playground embed      | iframe or direct WASM embed in the landing page                          |
| Documentation         | Markdown files, static rendering, searchable via Pagefind                |
| Fonts                 | System font stack -- no Google Fonts, no external requests on load       |

# **41.2 Repository Structure**

| **lumina-website/ directory structure**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| lumina-website/<br><br>src/<br><br>index.html -- landing page with embedded playground<br><br>why.html -- the problem Lumina solves<br><br>docs/ -- all documentation Markdown files<br><br>guides/ -- the four getting started guides<br><br>errors/ -- error code reference L001-L042<br><br>changelog.html -- version history<br><br>public/<br><br>lumina-wasm/ -- WASM binary and JS bindings (copied from lumina-wasm)<br><br>playground/ -- playground JS/CSS (copied from crates/lumina-wasm/pkg)<br><br>fonts/ -- self-hosted fonts if any<br><br>lumina-icon.png -- brand icon<br><br>deploy.sh -- copy latest WASM build into public/<br><br>package.json -- build scripts |

# **41.3 Landing Page -- Above the Fold HTML**

| **index.html -- the critical first section**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| &lt;!DOCTYPE html&gt;<br><br>&lt;html lang="en"&gt;<br><br>&lt;head&gt;<br><br>&lt;meta charset="UTF-8"&gt;<br><br>&lt;meta name="viewport" content="width=device-width, initial-scale=1.0"&gt;<br><br>&lt;title&gt;Lumina -- Describe what is true.&lt;/title&gt;<br><br>&lt;meta name="description" content="Declarative reactive language for IoT and infrastructure monitoring."&gt;<br><br>&lt;link rel="stylesheet" href="/css/main.css"&gt;<br><br>&lt;/head&gt;<br><br>&lt;body&gt;<br><br>&lt;header&gt;<br><br>&lt;span class="logo"&gt;Lumina&lt;/span&gt;<br><br>&lt;nav&gt;<br><br>&lt;a href="/docs"&gt;Docs&lt;/a&gt;<br><br>&lt;a href="/guides"&gt;Get Started&lt;/a&gt;<br><br>&lt;a href="<https://github.com/luminalang/lumina">GitHub</a>&gt;<br><br>&lt;/nav&gt;<br><br>&lt;/header&gt;<br><br>&lt;!-- Hero: tagline + live playground side by side --&gt;<br><br>&lt;section class="hero"&gt;<br><br>&lt;div class="hero-text"&gt;<br><br>&lt;h1&gt;Describe what is true.&lt;/h1&gt;<br><br>&lt;p&gt;Lumina is a declarative reactive language for IoT,<br><br>infrastructure monitoring, and real-time automation.&lt;/p&gt;<br><br>&lt;p&gt;Write rules that react to the world. No event loops.<br><br>No polling. No state machines.&lt;/p&gt;<br><br>&lt;div class="hero-actions"&gt;<br><br>&lt;a href="/guides/fleet" class="btn-primary"&gt;Get Started in 10 Minutes&lt;/a&gt;<br><br>&lt;a href="<https://lumina-lang.dev/play>" class="btn-secondary"&gt;Try in Browser&lt;/a&gt;<br><br>&lt;/div&gt;<br><br>&lt;/div&gt;<br><br>&lt;div class="hero-playground"&gt;<br><br>&lt;!-- Playground iframe -- pre-loaded with fleet example --&gt;<br><br>&lt;iframe src="/play?example=fleet" title="Lumina Playground"&gt;&lt;/iframe&gt;<br><br>&lt;/div&gt;<br><br>&lt;/section&gt;<br><br>&lt;/body&gt;<br><br>&lt;/html&gt; |

# **41.4 Deployment Pipeline**

| **.github/workflows/deploy-website.yml**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| name: Deploy Website<br><br>on:<br><br>push:<br><br>branches: \[main\]<br><br>workflow_dispatch:<br><br>jobs:<br><br>deploy:<br><br>runs-on: ubuntu-latest<br><br>steps:<br><br>\- uses: actions/checkout@v4<br><br>\# Build WASM<br><br>\- name: Install wasm-pack<br><br>run: curl <https://rustwasm.github.io/wasm-pack/installer/init.sh> -sSf \| sh<br><br>\- name: Build WASM<br><br>run: wasm-pack build crates/lumina-wasm --target web --out-dir ../../lumina-website/public/lumina-wasm<br><br>\# Deploy to Cloudflare Pages<br><br>\- name: Deploy to Cloudflare Pages<br><br>uses: cloudflare/pages-action@v1<br><br>with:<br><br>apiToken: \${{ secrets.CLOUDFLARE_API_TOKEN }}<br><br>accountId: \${{ secrets.CLOUDFLARE_ACCOUNT_ID }}<br><br>projectName: lumina-lang<br><br>directory: lumina-website |

# **41.5 Build Order**

**BUILD Chapter 41 -- exact sequence**

Step 1: Register lumina-lang.dev domain.

Step 2: Create lumina-website/ repository or directory in monorepo.

Step 3: Build landing page HTML/CSS -- hero section with playground iframe.

Step 4: Build why page with Python vs Lumina comparison.

Step 5: Set up Cloudflare Pages or Netlify deployment.

Step 6: Configure deploy.sh to copy latest WASM build into public/.

Step 7: Set up GitHub Actions workflow for automatic deployment on push.

Step 8: Add documentation structure -- empty pages are fine at launch.

Step 9: Deploy and verify lumina-lang.dev loads in under 2 seconds.

Step 10: Verify playground iframe loads and the fleet example runs.

**Chapter 42**

**The One-Line Installer**

_install.sh -- bulletproof, fast, silent, works first time every time_

# **42.1 GitHub Releases Pipeline**

Before the installer can work, binaries must be available on GitHub releases. This requires a release workflow that builds binaries for all platforms and uploads them.

| **.github/workflows/release.yml -- binary release pipeline**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| name: Release<br><br>on:<br><br>push:<br><br>tags: \["v\*"\]<br><br>jobs:<br><br>build:<br><br>strategy:<br><br>matrix:<br><br>include:<br><br>\- os: ubuntu-latest<br><br>target: x86_64-unknown-linux-gnu<br><br>binary: lumina-linux-x64<br><br>\- os: ubuntu-latest<br><br>target: aarch64-unknown-linux-gnu<br><br>binary: lumina-linux-arm64<br><br>\- os: macos-latest<br><br>target: aarch64-apple-darwin<br><br>binary: lumina-macos-arm64<br><br>\- os: macos-latest<br><br>target: x86_64-apple-darwin<br><br>binary: lumina-macos-x64<br><br>\- os: windows-latest<br><br>target: x86_64-pc-windows-msvc<br><br>binary: lumina-windows-x64.exe<br><br>runs-on: \${{ matrix.os }}<br><br>steps:<br><br>\- uses: actions/checkout@v4<br><br>\- name: Install Rust target<br><br>run: rustup target add \${{ matrix.target }}<br><br>\- name: Build lumina<br><br>run: cargo build --release --target \${{ matrix.target }} -p lumina-cli<br><br>\- name: Build lumina-lsp<br><br>run: cargo build --release --target \${{ matrix.target }} -p lumina-lsp<br><br>\- name: Rename and checksum<br><br>run: \|<br><br>cp target/\${{ matrix.target }}/release/lumina \${{ matrix.binary }}<br><br>cp target/\${{ matrix.target }}/release/lumina-lsp \${{ matrix.binary }}-lsp<br><br>sha256sum \${{ matrix.binary }} > \${{ matrix.binary }}.sha256<br><br>\- name: Upload to release<br><br>uses: softprops/action-gh-release@v1<br><br>with:<br><br>files: \|<br><br>\${{ matrix.binary }}<br><br>\${{ matrix.binary }}-lsp<br><br>\${{ matrix.binary }}.sha256 |

# **42.2 The Installer Script**

| **public/install.sh -- hosted at lumina-lang.dev/install.sh**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| #!/bin/sh<br><br>\# Lumina installer -- lumina-lang.dev<br><br>\# Usage: curl -fsSL <https://lumina-lang.dev/install.sh> \| sh<br><br>set -e<br><br>LUMINA_HOME="\${LUMINA_HOME:-\$HOME/.lumina}"<br><br>BIN_DIR="\$LUMINA_HOME/bin"<br><br>RELEASES="<https://github.com/luminalang/lumina/releases/latest/download>"<br><br>\# ── Detect platform ─────────────────────────────────────<br><br>detect_platform() {<br><br>OS=\$(uname -s)<br><br>ARCH=\$(uname -m)<br><br>case "\$OS" in<br><br>Linux)<br><br>case "\$ARCH" in<br><br>x86_64) echo "linux-x64" ;;<br><br>aarch64) echo "linux-arm64" ;;<br><br>\*) echo ""; return 1 ;;<br><br>esac ;;<br><br>Darwin)<br><br>case "\$ARCH" in<br><br>arm64) echo "macos-arm64" ;;<br><br>x86_64) echo "macos-x64" ;;<br><br>\*) echo ""; return 1 ;;<br><br>esac ;;<br><br>\*) echo ""; return 1 ;;<br><br>esac<br><br>}<br><br>PLATFORM=\$(detect_platform)<br><br>if \[ -z "\$PLATFORM" \]; then<br><br>echo "Error: unsupported platform. Download manually from:"<br><br>echo " <https://github.com/luminalang/lumina/releases>"<br><br>exit 1<br><br>fi<br><br>\# ── Download ────────────────────────────────────────────<br><br>echo "Installing Lumina for \$PLATFORM..."<br><br>mkdir -p "\$BIN_DIR"<br><br>download_binary() {<br><br>NAME="\$1"<br><br>URL="\$RELEASES/lumina-\$PLATFORM"<br><br>if \[ "\$NAME" = "lsp" \]; then<br><br>URL="\$RELEASES/lumina-lsp-\$PLATFORM"<br><br>DEST="\$BIN_DIR/lumina-lsp"<br><br>else<br><br>DEST="\$BIN_DIR/lumina"<br><br>fi<br><br>curl -fsSL "\$URL" -o "\$DEST"<br><br>curl -fsSL "\$URL.sha256" -o "\$DEST.sha256"<br><br>\# Verify checksum<br><br>EXPECTED=\$(cat "\$DEST.sha256" \| cut -d" " -f1)<br><br>ACTUAL=\$(sha256sum "\$DEST" 2>/dev/null \| cut -d" " -f1 \| shasum -a 256 "\$DEST" \| cut -d" " -f1)<br><br>if \[ "\$EXPECTED" != "\$ACTUAL" \]; then<br><br>echo "Error: checksum verification failed for \$DEST"<br><br>rm -f "\$DEST" "\$DEST.sha256"<br><br>exit 1<br><br>fi<br><br>rm -f "\$DEST.sha256"<br><br>chmod +x "\$DEST"<br><br>}<br><br>download_binary lumina<br><br>download_binary lsp<br><br>\# ── Add to PATH ─────────────────────────────────────────<br><br>add_to_path() {<br><br>EXPORT_LINE="export PATH="\$BIN_DIR:\$PATH""<br><br>for PROFILE in "\$HOME/.zshrc" "\$HOME/.bashrc" "\$HOME/.bash_profile" "\$HOME/.profile"; do<br><br>if \[ -f "\$PROFILE" \]; then<br><br>if ! grep -q "\$BIN_DIR" "\$PROFILE" 2>/dev/null; then<br><br>echo "\$EXPORT_LINE" >> "\$PROFILE"<br><br>fi<br><br>break<br><br>fi<br><br>done<br><br>export PATH="\$BIN_DIR:\$PATH"<br><br>}<br><br>add_to_path<br><br>\# ── Verify ──────────────────────────────────────────────<br><br>if lumina --version > /dev/null 2>&1; then<br><br>VERSION=\$(lumina --version)<br><br>echo "Lumina \$VERSION installed successfully."<br><br>echo ""<br><br>echo " Run: lumina run your-program.lum"<br><br>echo " Check: lumina check your-program.lum"<br><br>echo " Docs: <https://lumina-lang.dev/docs>"<br><br>echo ""<br><br>echo "Restart your terminal or run: source ~/.zshrc"<br><br>else<br><br>echo "Installation complete. Restart your terminal to use lumina."<br><br>fi |

# **42.3 Build Order**

**BUILD Chapter 42 -- exact sequence**

Step 1: Set up GitHub releases workflow (.github/workflows/release.yml).

Step 2: Create a test tag (v1.7.0-rc1) and verify all 5 platform binaries build.

Step 3: Verify checksum files are generated alongside each binary.

Step 4: Write install.sh with platform detection, download, checksum, PATH update.

Step 5: Host install.sh at lumina-lang.dev/install.sh (via static site public/).

Step 6: Test on macOS arm64 -- fresh machine, run curl install, verify lumina --version works.

Step 7: Test on Ubuntu x64 -- fresh machine, same test.

Step 8: Test failure case -- verify bad checksum produces clear error and cleans up.

Step 9: Test idempotency -- running installer twice must not break anything.

Step 10: Test PATH -- open new terminal after install, lumina must be on PATH.

**Chapter 43**

**Package Manager Support**

_Homebrew first, binaries always, APT and Winget to follow_

# **43.1 Homebrew -- macOS**

Homebrew is the highest priority package manager. Most macOS developers have it. The formula is a single Ruby file submitted to a tap repository.

| **Formula/lumina.rb -- complete Homebrew formula**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| class Lumina < Formula<br><br>desc "Declarative reactive language for IoT and infrastructure monitoring"<br><br>homepage "<https://lumina-lang.dev>"<br><br>license "MIT"<br><br>on_macos do<br><br>if Hardware::CPU.arm?<br><br>url "<https://github.com/luminalang/lumina/releases/download/v#{version}/lumina-v#{version}-macos-arm64.tar.gz>"<br><br>sha256 "FILL_IN_ON_RELEASE"<br><br>else<br><br>url "<https://github.com/luminalang/lumina/releases/download/v#{version}/lumina-v#{version}-macos-x64.tar.gz>"<br><br>sha256 "FILL_IN_ON_RELEASE"<br><br>end<br><br>end<br><br>def install<br><br>bin.install "lumina"<br><br>bin.install "lumina-lsp"<br><br>end<br><br>def caveats<br><br><<~EOS<br><br>lumina-lsp is installed alongside lumina.<br><br>Add it to VS Code by installing the Lumina extension:<br><br><https://marketplace.visualstudio.com/items?itemName=luminalang.lumina><br><br>EOS<br><br>end<br><br>test do<br><br>system "#{bin}/lumina", "--version"<br><br>system "#{bin}/lumina-lsp", "--version"<br><br>end<br><br>end |

| **Release workflow addition -- create tarballs for Homebrew**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \# Add to release.yml after building binaries:<br><br>\- name: Create macOS tarballs for Homebrew<br><br>if: matrix.os == "macos-latest"<br><br>run: \|<br><br>mkdir lumina-\${{ matrix.binary }}<br><br>cp target/\${{ matrix.target }}/release/lumina lumina-\${{ matrix.binary }}/<br><br>cp target/\${{ matrix.target }}/release/lumina-lsp lumina-\${{ matrix.binary }}/<br><br>tar czf lumina-v\${{ github.ref_name }}-\${{ matrix.binary }}.tar.gz lumina-\${{ matrix.binary }}/<br><br>sha256sum lumina-v\${{ github.ref_name }}-\${{ matrix.binary }}.tar.gz > lumina-v\${{ github.ref_name }}-\${{ matrix.binary }}.tar.gz.sha256 |

# **43.2 Homebrew Tap Setup**

| **How to set up the Lumina Homebrew tap**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \# 1. Create repository: github.com/luminalang/homebrew-lumina<br><br>\# Directory structure:<br><br>\# homebrew-lumina/<br><br>\# Formula/<br><br>\# lumina.rb<br><br>\# 2. Engineers install via:<br><br>brew tap luminalang/lumina<br><br>brew install luminalang/lumina/lumina<br><br>\# 3. Or shorter after tap is submitted to homebrew-core:<br><br>brew install lumina<br><br>\# NOTE: Submission to homebrew-core requires:<br><br>\# - 75+ GitHub stars<br><br>\# - Stable release with no pre-release versions<br><br>\# - Formula passing all brew audit checks<br><br>\# Start with the tap, submit to core after v2 launch |

# **43.3 Direct Binary Downloads Page**

| **docs/install.md -- binary downloads documentation**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| \# Manual Installation<br><br>\## Download the binary for your platform<br><br>\| Platform \| Binary \|<br><br>\|----------\|--------\|<br><br>\| macOS Apple Silicon \| lumina-macos-arm64 \|<br><br>\| macOS Intel \| lumina-macos-x64 \|<br><br>\| Linux x64 \| lumina-linux-x64 \|<br><br>\| Linux ARM64 \| lumina-linux-arm64 \|<br><br>\| Windows x64 \| lumina-windows-x64.exe \|<br><br>\## After downloading<br><br>\`\`\`bash<br><br>\# macOS / Linux<br><br>chmod +x lumina-linux-x64<br><br>mv lumina-linux-x64 ~/.local/bin/lumina<br><br>\# Verify<br><br>lumina --version<br><br>\`\`\`<br><br>\## Verify checksum<br><br>\`\`\`bash<br><br>sha256sum lumina-linux-x64<br><br>\# Compare with lumina-linux-x64.sha256 from the same release<br><br>\`\`\` |

# **43.4 Build Order**

**BUILD Chapter 43 -- exact sequence**

Step 1: Create github.com/luminalang/homebrew-lumina repository.

Step 2: Add Formula/lumina.rb with the formula content above.

Step 3: Update release.yml to create tarballs and compute SHA256 for Homebrew.

Step 4: On v1.7 release: update lumina.rb with correct sha256 values.

Step 5: Test: brew tap luminalang/lumina && brew install luminalang/lumina/lumina.

Step 6: Verify lumina --version and lumina-lsp --version both work after brew install.

Step 7: Add binary download section to website install page.

Step 8: APT and Winget -- defer to v1.8 or when community requests them.

**Chapter 44**

**VS Code Marketplace**

_vsce publish -- from .vsix file to searchable marketplace extension_

# **44.1 Prerequisites**

| **Requirement**       | **How to get it**                                                                        |
| --------------------- | ---------------------------------------------------------------------------------------- |
| Azure DevOps account  | Create at dev.azure.com -- free                                                          |
| Publisher account     | Register at marketplace.visualstudio.com/manage -- use "luminalang"                      |
| Personal Access Token | Azure DevOps > User Settings > Personal Access Tokens > New Token > Marketplace (Manage) |
| vsce CLI              | npm install -g @vscode/vsce                                                              |
| Extension icon        | lumina-icon.png -- 128x128px minimum                                                     |

# **44.2 package.json -- Complete Marketplace Metadata**

| **extensions/lumina-vscode/package.json -- complete v1.7 version**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| {<br><br>"name": "lumina-lang",<br><br>"displayName": "Lumina",<br><br>"description": "Declarative reactive language for IoT and infrastructure monitoring",<br><br>"version": "1.7.0",<br><br>"publisher": "luminalang",<br><br>"license": "MIT",<br><br>"categories": \["Programming Languages", "Linters"\],<br><br>"keywords": \["lumina", "reactive", "IoT", "monitoring", "declarative", "infrastructure"\],<br><br>"icon": "images/lumina-icon.png",<br><br>"homepage": "<https://lumina-lang.dev>",<br><br>"repository": {<br><br>"type": "git",<br><br>"url": "<https://github.com/luminalang/lumina>"<br><br>},<br><br>"bugs": { "url": "<https://github.com/luminalang/lumina/issues>" },<br><br>"engines": { "vscode": "^1.75.0" },<br><br>"main": "./out/extension.js",<br><br>"activationEvents": \["onLanguage:lumina"\],<br><br>"contributes": {<br><br>"languages": \[{<br><br>"id": "lumina",<br><br>"aliases": \["Lumina", "lumina"\],<br><br>"extensions": \[".lum"\],<br><br>"configuration": "./language-configuration.json"<br><br>}\],<br><br>"grammars": \[{<br><br>"language": "lumina",<br><br>"scopeName": "source.lumina",<br><br>"path": "./syntaxes/lumina.tmLanguage.json"<br><br>}\],<br><br>"snippets": \[{<br><br>"language": "lumina",<br><br>"path": "./snippets/lumina.json"<br><br>}\]<br><br>},<br><br>"scripts": {<br><br>"compile": "tsc -p ./",<br><br>"package": "vsce package",<br><br>"publish": "vsce publish"<br><br>},<br><br>"dependencies": {<br><br>"vscode-languageclient": "^9.0.0"<br><br>},<br><br>"devDependencies": {<br><br>"@types/vscode": "^1.75.0",<br><br>"@vscode/vsce": "^2.0.0",<br><br>"typescript": "^5.0.0"<br><br>}<br><br>} |

# **44.3 Automated Marketplace Publication**

| **.github/workflows/publish-extension.yml**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| name: Publish VS Code Extension<br><br>on:<br><br>push:<br><br>tags: \["v\*"\]<br><br>jobs:<br><br>publish:<br><br>runs-on: ubuntu-latest<br><br>steps:<br><br>\- uses: actions/checkout@v4<br><br>\- uses: actions/setup-node@v4<br><br>with:<br><br>node-version: "20"<br><br>\- name: Install dependencies<br><br>run: cd extensions/lumina-vscode && npm install<br><br>\- name: Compile TypeScript<br><br>run: cd extensions/lumina-vscode && npm run compile<br><br>\- name: Publish to marketplace<br><br>run: cd extensions/lumina-vscode && npx vsce publish<br><br>env:<br><br>VSCE_PAT: \${{ secrets.VSCE_PAT }} |

# **44.4 Build Order**

**BUILD Chapter 44 -- exact sequence**

Step 1: Create Azure DevOps account and publisher account under "luminalang".

Step 2: Generate Personal Access Token with Marketplace publish scope.

Step 3: Add VSCE_PAT as GitHub Actions secret.

Step 4: Update package.json with all marketplace metadata from above.

Step 5: Add lumina-icon.png (128x128px) to extensions/lumina-vscode/images/.

Step 6: npm install && npm run compile -- must succeed with 0 TypeScript errors.

Step 7: vsce package -- produces lumina-lang-1.7.0.vsix.

Step 8: vsce publish -- publishes to marketplace.

Step 9: Open VS Code > Extensions > search "Lumina" -- verify it appears.

Step 10: Install from marketplace on fresh VS Code -- verify LSP starts automatically.

Step 11: Open a .lum file -- verify diagnostics, hover, and completion all work.

Step 12: Add publish workflow to GitHub Actions for automatic publish on release tags.

**Chapter 45**

**Getting Started Guides**

_Four Markdown guides -- one flagship, three secondary -- hosted on lumina-lang.dev_

# **45.1 Guide File Structure**

| **lumina-website/src/guides/ directory**                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| guides/<br><br>index.html -- guide selector page<br><br>fleet/ -- FLAGSHIP: Delivery Fleet Monitoring<br><br>index.md<br><br>fleet-complete.lum -- the complete program engineers can download<br><br>sensors/ -- Temperature Sensor Network<br><br>index.md<br><br>sensors-complete.lum<br><br>datacenter/ -- Data Center Basic Monitoring<br><br>index.md<br><br>datacenter-complete.lum<br><br>agriculture/ -- Smart Agriculture Soil Sensors<br><br>index.md<br><br>agriculture-complete.lum |

# **45.2 Guide Template Structure**

| **guides/fleet/index.md -- guide structure template**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \---<br><br>title: "Monitor a Delivery Fleet in 10 Minutes"<br><br>domain: "Delivery and Logistics"<br><br>time: "10 minutes"<br><br>features: \["entities", "derived fields", "rules", "aggregate", "alert", "cooldown"\]<br><br>\---<br><br>\# Monitor a Delivery Fleet in 10 Minutes<br><br>\## The Problem<br><br>You have a fleet of delivery motos. Each one has a battery.<br><br>When a battery drops below 20% you need to know immediately.<br><br>When the whole fleet goes offline -- you need a critical alert.<br><br>Today you write 200+ lines of Python: polling loops, state flags,<br><br>cooldown timers, alert deduplication. All of it fragile.<br><br>In Lumina: 30 lines. No loops. No flags. Just truth.<br><br>\## What You Will Build<br><br>\[screenshot or animated gif of the playground showing the program running\]<br><br>\## The Complete Program<br><br>\`\`\`lumina<br><br>\[full program here -- no abbreviations\]<br><br>\`\`\`<br><br>\## Line by Line<br><br>\### The entity -- what a moto IS<br><br>\[explanation of entity block\]<br><br>\### The aggregate -- what the fleet IS<br><br>\[explanation of aggregate block\]<br><br>\### The rules -- what should HAPPEN<br><br>\[explanation of each rule\]<br><br>\## Running It<br><br>\`\`\`bash<br><br>lumina run fleet.lum<br><br>\`\`\`<br><br>\## What You Should See<br><br>\[exact expected output\]<br><br>\## Common Errors<br><br>\*\*L001: Unknown entity "Moto"\*\*<br><br>\[explanation and fix\]<br><br>\*\*L028: Invalid severity level\*\*<br><br>\[explanation and fix\]<br><br>\*\*L034: Cooldown duration is zero\*\*<br><br>\[explanation and fix\]<br><br>\## What to Build Next<br><br>\- Add external entity to connect to real MQTT data<br><br>\- Add prev() to detect battery drain rate<br><br>\- Try the \[Data Center guide\](/guides/datacenter) |

# **45.3 Guide Quality Verification**

**RULE Before publishing any guide -- verify all of these**

A developer with no Lumina experience can complete it in under 10 minutes.

Every line of code in the guide is explained.

The program is complete and correct -- no "..." or placeholder sections.

Running lumina run program.lum produces the exact output shown in the guide.

All three listed errors actually occur when the guide says they do.

The fix for each error is correct and complete.

The guide works on macOS, Linux, and Windows.

The complete .lum file is available for download.

# **45.4 Build Order**

**BUILD Chapter 45 -- exact sequence**

Step 1: Write the flagship fleet guide first -- it must be perfect before any other.

Step 2: Test the flagship guide with someone who has never seen Lumina.

Step 3: Fix everything they found confusing -- no matter how small.

Step 4: Write sensor network guide.

Step 5: Write data center guide.

Step 6: Write agriculture guide.

Step 7: Add all four .lum files as downloadable assets.

Step 8: Add guide selector page at /guides/.

Step 9: Link from website landing page to flagship guide.

Step 10: Link each guide to the playground with the example pre-loaded.

**Chapter 46**

**Error Message Review**

_Every L001-L042 rewritten to teach -- the most underrated v1.7 chapter_

# **46.1 The Review Process**

Every error code is reviewed against the three-question standard: What went wrong? Why does Lumina not allow this? How do I fix it? Any error that does not answer all three questions is rewritten.

| **Category**               | **Error codes** | **Focus of review**                                   |
| -------------------------- | --------------- | ----------------------------------------------------- |
| Core entity errors         | L001-L010       | Improve suggestions -- show available entities/fields |
| Type and expression errors | L011-L018       | Show expected vs actual type clearly                  |
| Rule and trigger errors    | L019-L023       | Explain becomes/for/every semantics                   |
| v1.5 prev errors           | L024-L025       | Add example of correct prev() usage                   |
| v1.5 fleet errors          | L026-L027       | Explain when any/all Boolean requirement              |
| v1.5 alert errors          | L028-L030       | List valid severity levels explicitly                 |
| v1.5 aggregate errors      | L031-L033       | Show which aggregate functions need Number vs Boolean |
| v1.5 cooldown error        | L034            | Show minimum valid cooldown duration                  |
| v1.6 compound errors       | L035            | Show 3-condition maximum with example                 |
| v1.6 ref errors            | L036-L037       | Show correct ref syntax and cycle example             |
| v1.6 write error           | L038            | Show update vs write distinction clearly              |
| v1.6 frequency errors      | L039-L040       | Show minimum count=2 and positive window              |
| v1.6 timestamp errors      | L041-L042       | Explain now() restriction and .age comparison         |

# **46.2 Error Format Specification**

| **Standard error message format for all L-codes**                                                                                                                                                                                                                                                                      |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| L0XX: \[one sentence -- what went wrong\]<br><br>\[the offending line with caret pointing to the problem\]<br><br>\[ ^^^^^^^^^^^ \]<br><br>"\[field/entity name\]" is \[what it is\].<br><br>\[why Lumina does not allow this -- one sentence\].<br><br>Did you mean: \[concrete fix suggestion or correct example\] ? |

| **L036 -- example of the new format**                                                                                                                                                                                                                                                                                       |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| L036: ref target entity does not exist.<br><br>entity Server {<br><br>cooling: ref CoolingSystem<br><br>^^^^^^^^^^^^<br><br>"CoolingSystem" is not a declared entity.<br><br>ref requires the target entity to be declared before use.<br><br>Did you mean: "CoolingUnit" ?<br><br>Or declare: entity CoolingSystem { ... } |

# **46.3 Implementation Location**

| **Where error messages live in the codebase**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Error messages are produced in:<br><br>\-- crates/lumina-diagnostics/src/messages.rs<br><br>\-- or equivalent diagnostic message file<br><br>\-- Each L-code maps to a format string:<br><br>\-- fn l036_message(target: &str, suggestion: Option&lt;&str&gt;) -> String {<br><br>\-- format!("L036: ref target entity does not exist.\\n\\n ...", target, suggestion)<br><br>\-- }<br><br>\-- The review in v1.7 rewrites these format strings.<br><br>\-- Do NOT change the error code numbers.<br><br>\-- Do NOT change where errors are emitted.<br><br>\-- ONLY change the message text to meet the three-question standard. |

# **46.4 Build Order**

**BUILD Chapter 46 -- exact sequence**

Step 1: Find where error messages are defined in lumina-diagnostics.

Step 2: For each L001-L042: read current message, apply three-question test.

Step 3: Rewrite any message that fails the test. Use the format from 46.2.

Step 4: For L001-L010: add name similarity suggestions (Levenshtein distance or similar).

Step 5: cargo test --workspace -- existing tests must still pass.

Step 6: Write the error reference page on the website -- one section per error code.

Step 7: Test five most common errors by intentionally triggering them in a .lum file.

Step 8: Verify each error: is the message clear? Does it answer all three questions?

**Chapter 47**

**Docker Image**

_luminalang/runtime -- the path for infrastructure engineers and CI/CD pipelines_

# **47.1 The Dockerfile**

| **docker/Dockerfile -- the production image**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \# ── Build stage ─────────────────────────────────────────<br><br>FROM rust:1.75-slim AS builder<br><br>WORKDIR /build<br><br>COPY . .<br><br>RUN cargo build --release -p lumina-cli -p lumina-lsp<br><br>\# ── Runtime stage ────────────────────────────────────────<br><br>FROM debian:bookworm-slim<br><br>RUN apt-get update && apt-get install -y \\<br><br>ca-certificates \\<br><br>&& rm -rf /var/lib/apt/lists/\*<br><br>COPY --from=builder /build/target/release/lumina /usr/local/bin/lumina<br><br>COPY --from=builder /build/target/release/lumina-lsp /usr/local/bin/lumina-lsp<br><br>RUN chmod +x /usr/local/bin/lumina /usr/local/bin/lumina-lsp<br><br>WORKDIR /workspace<br><br>ENTRYPOINT \["lumina"\]<br><br>CMD \["--help"\]<br><br>\# ── Labels ───────────────────────────────────────────────<br><br>LABEL org.opencontainers.image.title="Lumina Runtime"<br><br>LABEL org.opencontainers.image.description="Declarative reactive language for IoT and infrastructure"<br><br>LABEL org.opencontainers.image.url="<https://lumina-lang.dev>"<br><br>LABEL org.opencontainers.image.source="<https://github.com/luminalang/lumina>" |

# **47.2 Docker Hub Publication Workflow**

| **.github/workflows/docker-publish.yml**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| name: Publish Docker Image<br><br>on:<br><br>push:<br><br>tags: \["v\*"\]<br><br>jobs:<br><br>docker:<br><br>runs-on: ubuntu-latest<br><br>steps:<br><br>\- uses: actions/checkout@v4<br><br>\- name: Log in to Docker Hub<br><br>uses: docker/login-action@v3<br><br>with:<br><br>username: \${{ secrets.DOCKERHUB_USERNAME }}<br><br>password: \${{ secrets.DOCKERHUB_TOKEN }}<br><br>\- name: Extract version<br><br>id: version<br><br>run: echo "version=\${GITHUB_REF#refs/tags/v}" >> \$GITHUB_OUTPUT<br><br>\- name: Build and push<br><br>uses: docker/build-push-action@v5<br><br>with:<br><br>context: .<br><br>file: docker/Dockerfile<br><br>push: true<br><br>tags: \|<br><br>luminalang/runtime:latest<br><br>luminalang/runtime:\${{ steps.version.outputs.version }}<br><br>platforms: linux/amd64,linux/arm64 |

# **47.3 Build Order**

**BUILD Chapter 47 -- exact sequence**

Step 1: Create Docker Hub account under "luminalang".

Step 2: Create DOCKERHUB_USERNAME and DOCKERHUB_TOKEN secrets in GitHub.

Step 3: Write docker/Dockerfile using the multi-stage build above.

Step 4: docker build -t luminalang/runtime:test . -- must succeed locally.

Step 5: docker run luminalang/runtime:test --version -- must print version.

Step 6: docker run -v \$(pwd):/workspace luminalang/runtime:test run test.lum -- must run.

Step 7: Add Docker Hub publication workflow to GitHub Actions.

Step 8: Tag a release -- verify image appears on hub.docker.com/r/luminalang/runtime.

Step 9: Add Docker usage section to website install page.

Step 10: Write docker-compose example showing Lumina with MQTT broker.

**Chapter 48**

**Playground Polish**

_Performance, pre-loaded examples, mobile layout, error display, share URL reliability_

# **48.1 WASM Binary Size Reduction**

| **Cargo.toml -- release profile optimization for size**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[profile.release\]<br><br>opt-level = "z" # Optimize for size<br><br>lto = true # Link-time optimization<br><br>codegen-units = 1 # Single codegen unit for better optimization<br><br>panic = "abort" # Smaller panic handler<br><br>strip = true # Strip debug symbols<br><br>\# In crates/lumina-wasm/Cargo.toml:<br><br>\[dependencies\]<br><br>\# Remove any dependency not strictly needed for WASM<br><br>\# serde_json can be feature-gated<br><br>\# logging can be stripped in WASM builds<br><br>\# Build command for minimum size:<br><br>\# wasm-pack build crates/lumina-wasm --target web --release<br><br>\# wasm-opt -Oz -o pkg/lumina_wasm_bg.wasm pkg/lumina_wasm_bg.wasm<br><br>\# Target: under 500KB gzipped<br><br>\# Measure with: gzip -c pkg/lumina_wasm_bg.wasm \| wc -c |

# **48.2 Pre-Loaded Playground State**

| **playground/src/examples.ts -- pre-loaded example programs**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| export const EXAMPLES = {<br><br>fleet: {<br><br>name: "Delivery Fleet",<br><br>description: "Monitor a fleet of delivery motos",<br><br>source: \`<br><br>entity Moto {<br><br>battery: Number<br><br>isOnline: Boolean<br><br>label: Text<br><br>isLowBattery := battery < 20<br><br>}<br><br>aggregate FleetStatus over Moto {<br><br>avgBattery := avg(battery)<br><br>onlineCount := count(isOnline)<br><br>anyLow := any(isLowBattery)<br><br>}<br><br>rule LowBattery for (m: Moto)<br><br>when m.isLowBattery becomes true cooldown 5m {<br><br>alert severity: "warning", source: m.label,<br><br>message: "low battery: {m.battery}%"<br><br>} on clear {<br><br>alert severity: "resolved", source: m.label,<br><br>message: "battery recovered"<br><br>}<br><br>let moto1 = Moto { battery: 80, isOnline: true, label: "moto-north-1" }<br><br>let moto2 = Moto { battery: 45, isOnline: true, label: "moto-north-2" }<br><br>let moto3 = Moto { battery: 12, isOnline: true, label: "moto-south-1" }<br><br>\`<br><br>},<br><br>sensors: { name: "Temperature Sensors", source: \`...\` },<br><br>datacenter: { name: "Data Center", source: \`...\` },<br><br>agriculture: { name: "Smart Agriculture", source: \`...\` },<br><br>};<br><br>export type ExampleKey = keyof typeof EXAMPLES; |

# **48.3 URL Parameter for Example Selection**

| **playground/src/App.tsx -- load example from URL**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| import { loadFromURL } from "./ShareButton";<br><br>import { EXAMPLES } from "./examples";<br><br>function getInitialSource(): string {<br><br>// 1. Check URL fragment for shared program<br><br>const shared = loadFromURL();<br><br>if (shared) return shared;<br><br>// 2. Check URL query param for example<br><br>const params = new URLSearchParams(window.location.search);<br><br>const example = params.get("example");<br><br>if (example && example in EXAMPLES) {<br><br>return EXAMPLES\[example as keyof typeof EXAMPLES\].source;<br><br>}<br><br>// 3. Default to fleet example<br><br>return EXAMPLES.fleet.source;<br><br>}<br><br>// Usage: /play?example=fleet loads fleet example<br><br>// Usage: /play?example=datacenter loads data center example<br><br>// Usage: /play#v=2&src=... loads shared program |

# **48.4 Mobile Responsive CSS**

| **playground/src/styles/playground.css -- mobile breakpoints**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| /\* Desktop: side-by-side editor and state panel \*/<br><br>.playground-layout {<br><br>display: grid;<br><br>grid-template-columns: 1fr 1fr;<br><br>gap: 16px;<br><br>height: 100vh;<br><br>}<br><br>/\* Tablet: stack vertically \*/<br><br>@media (max-width: 768px) {<br><br>.playground-layout {<br><br>grid-template-columns: 1fr;<br><br>grid-template-rows: 50vh auto;<br><br>}<br><br>.state-panel { max-height: 50vh; overflow-y: auto; }<br><br>}<br><br>/\* Mobile: editor full width, panels below \*/<br><br>@media (max-width: 480px) {<br><br>.playground-layout {<br><br>grid-template-columns: 1fr;<br><br>}<br><br>.monaco-editor { font-size: 12px; }<br><br>.card { padding: 8px; }<br><br>.timeline-entry { font-size: 11px; }<br><br>} |

# **48.5 Build Order**

**BUILD Chapter 48 -- exact sequence**

Step 1: Apply Cargo.toml release profile settings for WASM size optimization.

Step 2: Build WASM with wasm-opt -- measure gzipped size, iterate until under 500KB.

Step 3: Create playground/src/examples.ts with all four domain examples.

Step 4: Update App.tsx to load example from URL param or default to fleet.

Step 5: Add example selector dropdown to playground header.

Step 6: Apply mobile responsive CSS breakpoints.

Step 7: Test on iPhone SE viewport (375px) -- verify usable.

Step 8: Test share URL on Chrome, Firefox, Safari, Edge -- verify loads correctly.

Step 9: Performance test: load playground, measure time to interactive.

Step 10: Verify pre-loaded fleet example fires one alert immediately on load.

Step 11: Embed playground iframe in website landing page.

Step 12: Verify iframe loads correctly on lumina-lang.dev.

**Appendix**

**Complete v1.7 Build Sequence**

_8 chapters -- build in dependency order -- website launches when all are done_

**NOTE Prerequisite: v1.6 Complete**

cargo test --workspace must be 100% green.

lumina binary must be built and tested.

lumina-lsp binary must be built and tested.

WASM binary must be built with wasm-pack.

All error codes L001-L042 must be implemented.

Do NOT start v1.7 until all of the above are confirmed.

**BUILD Phase 1 -- Ch48: Playground Polish (first -- everything depends on it)**

1\. Optimize WASM binary size -- measure, iterate, confirm under 500KB gzipped.

2\. Create examples.ts with all four domain examples.

3\. Update App.tsx for URL-based example loading.

4\. Apply mobile responsive CSS.

5\. Test share URL on all major browsers.

6\. Verify pre-loaded fleet example fires alert on load.

**BUILD Phase 2 -- Ch42: Installer (second -- website links to it)**

1\. Set up GitHub releases workflow -- build all 5 platform binaries.

2\. Write install.sh with platform detection + checksum verification.

3\. Test on macOS arm64 and Linux x64 -- must work first time.

4\. Test failure case -- bad checksum must produce clear error.

**BUILD Phase 3 -- Ch43: Package Managers**

1\. Create homebrew-lumina tap repository.

2\. Write Formula/lumina.rb.

3\. Update release workflow to create tarballs.

4\. Test brew install on macOS.

5\. Add binary download page to website.

**BUILD Phase 4 -- Ch44: VS Code Marketplace**

1\. Set up publisher account and PAT.

2\. Update package.json with marketplace metadata.

3\. Add extension icon.

4\. vsce package && vsce publish.

5\. Verify extension appears in VS Code marketplace search.

6\. Set up automated publish workflow.

**BUILD Phase 5 -- Ch46: Error Message Review (independent -- can run in parallel)**

1\. Review all L001-L042 against three-question standard.

2\. Rewrite any that fail.

3\. cargo test --workspace -- must stay green.

4\. Write error reference page for website.

**BUILD Phase 6 -- Ch45: Getting Started Guides (needs playground + installer done first)**

1\. Write flagship fleet guide.

2\. Test with a developer who has never seen Lumina.

3\. Fix everything they found confusing.

4\. Write three secondary guides.

5\. Add all .lum downloadable files.

**BUILD Phase 7 -- Ch47: Docker Image (independent -- can run in parallel)**

1\. Write Dockerfile.

2\. Build and test locally.

3\. Set up Docker Hub and publish workflow.

4\. Verify image on Docker Hub.

**BUILD Phase 8 -- Ch41: Website (last -- depends on everything else being ready)**

1\. Register lumina-lang.dev domain.

2\. Build static site with landing page, docs, guides.

3\. Embed playground in hero section.

4\. Set up Cloudflare Pages deployment.

5\. Link to installer, marketplace, GitHub.

6\. Deploy and verify loads in under 2 seconds.

**DONE v1.7 Definition of Done -- 12 Verification Points**

1\. lumina-lang.dev loads in under 2 seconds.

2\. Playground embedded above the fold -- fleet example pre-loaded and reacting.

3\. curl install works on macOS and Linux -- lumina --version works after.

4\. brew install lumina works on macOS.

5\. Direct binary downloads available for all 5 platforms with checksums.

6\. VS Code extension searchable and installable from marketplace.

7\. Installing extension -- LSP starts -- diagnostics work in .lum files.

8\. Flagship fleet guide: complete in under 10 minutes, alert fires at end.

9\. All four domain guides published and tested.

10\. All L001-L042 error messages meet the three-question standard.

11\. Docker image available at luminalang/runtime:latest.

12\. Playground: WASM under 500KB gzipped, share URL works across all browsers.