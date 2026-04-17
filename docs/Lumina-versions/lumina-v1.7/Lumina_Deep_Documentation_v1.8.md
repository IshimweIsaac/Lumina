**LUMINA**

**v1.8 Deep Documentation**

**The Experience Release**

_Website | Installer | Package Managers | VS Code Marketplace | Getting Started Guides | Error Messages | Docker | Playground Polish_

_"Describe what is true. Lumina figures out what to do."_

_2026 | Chapters 41-48 | No new language features | The world gets ready for Lumina_

_Designed and authored by Isaac Ishimwe_

**Why v1.8**

**The Experience Release**

_Power without accessibility is a dead language -- v1.8 fixes that_

By the end of v1.6 Lumina can describe any reactive system. Entities, relationships, compound conditions, frequency patterns, write-back to the physical world, temporal truth. The language is complete. But complete and accessible are two different things.

v1.8 adds zero new language features. Every chapter is about the gap between "Lumina works" and "Lumina feels ready." That gap is smaller than it sounds but it matters enormously. The first engineer who encounters Lumina forms an opinion that is almost impossible to change. v1.8 makes sure that opinion is the right one.

**CORE The v1.8 Philosophy**

v1.6 made Lumina powerful. v1.8 makes Lumina accessible.

Power without adoption is a dead project.

v1.8 does not add a single new language feature.

v1.8 makes the language that already exists feel inevitable.

# **What v1.8 Is and Is Not**

| **v1.8 IS**                                     | **v1.8 IS NOT**                            |
| ----------------------------------------------- | ------------------------------------------ |
| The installer being built and tested            | A new language release                     |
| The website going live with playground embedded | New syntax or new runtime features         |
| The VS Code extension on the marketplace        | A replacement for v1.6                     |
| Four getting started guides written and tested  | A public release -- that is v2             |
| Every error message reviewed and improved       | A feature sprint                           |
| Docker image published                          | A marketing exercise                       |
| Playground polished to production quality       | A demo system separate from the playground |

# **The First Impression Journey**

Every engineer who discovers Lumina walks the same path. v1.8 makes every step of that path smooth.

| **Step** | **What happens**                                             | **v1.8 chapter**            |
| -------- | ------------------------------------------------------------ | --------------------------- |
| 1        | Hears about Lumina -- visits lumina-lang.dev                 | Ch41 Website                |
| 2        | Sees playground alive above the fold -- tries it immediately | Ch48 Playground Polish      |
| 3        | Interested -- runs the one-line installer                    | Ch42 Installer              |
| 4        | Opens VS Code -- searches Lumina -- installs extension       | Ch44 VS Code Marketplace    |
| 5        | Opens first .lum file -- LSP works immediately               | Ch44 + Ch38 LSP v2          |
| 6        | Follows the flagship getting started guide                   | Ch45 Getting Started Guides |
| 7        | Hits an error -- reads it -- fixes it immediately            | Ch46 Error Messages         |
| 8        | First real alert fires                                       | Ch45 Complete               |
| 9        | Shares it with a colleague                                   | Ch41 + Ch48 Share URL       |

That entire journey -- from first touch to first real program -- should take under 15 minutes. If any step breaks or confuses, the engineer drops off and does not come back. v1.8 exists to make every step work perfectly.

**Chapter 41**

**The Website -- lumina-lang.dev**

_The public home of Lumina -- simple, fast, and alive on arrival_

The website is not a marketing site. It is a window into what Lumina is. Engineers who hear about Lumina need somewhere to go. The website answers three questions in under 30 seconds: what is this, who is it for, and can I try it right now. Everything else is secondary.

# **41.1 Above the Fold -- The Only Thing That Matters**

The moment an engineer lands on lumina-lang.dev they should see Lumina reacting. Not a video. Not a screenshot. The actual playground, pre-loaded with a fleet monitoring program, already running. Battery sliders visible. An alert timeline already showing one fired alert.

The engineer did not click anything. Lumina is already alive. That is the wow moment. Not a demo system. Not a separate product. The playground itself -- embedded directly in the page, first thing visible, no scrolling required.

**NOTE The One Insight from ChatGPT Worth Keeping**

ChatGPT correctly identified that the website needs something alive above the fold.

The implementation is the embedded playground -- not a separate killer demo system.

The playground IS the killer demo. Making it excellent is Chapter 48.

Do not build a separate animated demo. Polish what already exists.

# **41.2 Page Structure**

| **Page / Section**   | **Purpose**                                                                                     |
| -------------------- | ----------------------------------------------------------------------------------------------- |
| Landing page -- hero | Embedded playground pre-loaded with fleet example. Philosophy tagline. Zero friction.           |
| What is Lumina       | One paragraph. The problem it solves. Who it is for. No jargon.                                 |
| Why Lumina           | Three concrete comparisons: Python monitoring code vs Lumina equivalent. Shows the clarity gap. |
| Domains              | Delivery fleets. Data centers. Agriculture. Healthcare. One sentence each. Links to guides.     |
| Documentation        | Links to all version docs. Searchable reference. Error code catalogue.                          |
| Getting Started      | Direct links to all four domain guides. Flagship delivery fleet guide first.                    |
| Changelog            | One paragraph per version. What it added and why.                                               |
| Email signup         | For engineers who want to know when public release happens at v2.                               |

# **41.3 The Philosophy Tagline**

One line. Visible immediately. Not a slogan -- the actual design principle.

| **The tagline that appears on the website**                                                                                                                                                           |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "Describe what is true. Lumina figures out what to do."<br><br>\-- Everything else on the website is an elaboration of this one line.<br><br>\-- If an engineer reads nothing else -- they read this. |

# **41.4 The Comparison Section -- Why This Wins Engineers**

The most powerful thing on the website is not a feature list. It is a side-by-side comparison showing what engineers currently write versus what Lumina lets them write. The clarity gap is immediately visible.

| **What engineers write today -- Python monitoring script**                                                                                                                                                                                                                                                                                                                                                |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| def check_fleet():<br><br>for device in devices:<br><br>if device.battery < threshold:<br><br>if not device.alert_sent:<br><br>if time.time() - device.last_alert > cooldown:<br><br>send_alert(device)<br><br>device.alert_sent = True<br><br>device.last_alert = time.time()<br><br>\# Plus: event loop, polling, state management, error handling...<br><br>\# 200+ lines to express what should be 10 |

| **What Lumina lets them write instead**                                                                                                                                                                                                                                                                                                                                                    |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| entity Moto {<br><br>battery: Number<br><br>isLowBattery := battery < 20<br><br>}<br><br>rule LowBattery for (m: Moto)<br><br>when m.isLowBattery becomes true<br><br>cooldown 5m {<br><br>alert severity: "warning", source: m.label,<br><br>message: "low battery: {m.battery}%"<br><br>}<br><br>\-- No event loop. No polling. No state management.<br><br>\-- Just truth and reaction. |

# **41.5 Technical Stack**

| **Decision**                              | **Reasoning**                                          |
| ----------------------------------------- | ------------------------------------------------------ |
| Static site -- no backend                 | Fast, cheap, no server maintenance, deploys from git   |
| Playground embedded via WASM              | The same runtime that runs locally runs in the browser |
| Markdown documentation                    | Easy to update, version controlled, searchable         |
| No JavaScript framework for the main site | Fast load times -- engineers leave slow sites          |
| lumina-lang.dev domain                    | Short, clear, professional                             |

**Chapter 42**

**The One-Line Installer**

_curl -fsSL <https://lumina-lang.dev/install.sh> | sh_

The installer is a commitment. The moment an engineer runs it they are trusting Lumina with their development environment. One failed install poisons the well permanently. The installer must be bulletproof, fast, and silent. No confusion. No manual steps. No PATH instructions. It just works.

# **42.1 What the Installer Does**

| **Step**                | **Action**                                                        |
| ----------------------- | ----------------------------------------------------------------- |
| 1\. Detect platform     | macOS arm64 / macOS x64 / Linux x64 / Windows x64                 |
| 2\. Download binary     | Fetches correct lumina binary from GitHub releases                |
| 3\. Download LSP binary | Fetches lumina-lsp binary from GitHub releases                    |
| 4\. Verify checksum     | SHA256 verification before installation                           |
| 5\. Place binaries      | Installs to ~/.lumina/bin/                                        |
| 6\. Update PATH         | Adds ~/.lumina/bin to shell profile (.zshrc / .bashrc / .profile) |
| 7\. Verify install      | Runs lumina --version to confirm success                          |
| 8\. Prompt VS Code      | Detects VS Code and prompts to install extension                  |

# **42.2 The Installer Script**

| **install.sh -- the complete installer**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| #!/bin/sh<br><br>set -e<br><br>LUMINA_VERSION="latest"<br><br>INSTALL_DIR="\$HOME/.lumina/bin"<br><br>RELEASES="<https://github.com/luminalang/lumina/releases>"<br><br>\# Detect platform<br><br>OS=\$(uname -s \| tr "\[:upper:\]" "\[:lower:\]")<br><br>ARCH=\$(uname -m)<br><br>case "\$ARCH" in<br><br>x86_64) ARCH="x64" ;;<br><br>arm64\|aarch64) ARCH="arm64" ;;<br><br>\*) echo "Unsupported architecture: \$ARCH"; exit 1 ;;<br><br>esac<br><br>BINARY="lumina-\${OS}-\${ARCH}"<br><br>LSP_BINARY="lumina-lsp-\${OS}-\${ARCH}"<br><br>\# Download<br><br>echo "Installing Lumina..."<br><br>mkdir -p "\$INSTALL_DIR"<br><br>curl -fsSL "\$RELEASES/latest/download/\$BINARY" -o "\$INSTALL_DIR/lumina"<br><br>curl -fsSL "\$RELEASES/latest/download/\$LSP_BINARY" -o "\$INSTALL_DIR/lumina-lsp"<br><br>chmod +x "\$INSTALL_DIR/lumina" "\$INSTALL_DIR/lumina-lsp"<br><br>\# Verify<br><br>ACTUAL=\$(sha256sum "\$INSTALL_DIR/lumina" \| cut -d" " -f1)<br><br>EXPECTED=\$(curl -fsSL "\$RELEASES/latest/download/\$BINARY.sha256")<br><br>if \[ "\$ACTUAL" != "\$EXPECTED" \]; then<br><br>echo "Checksum verification failed"; exit 1<br><br>fi<br><br>\# Add to PATH<br><br>PROFILE="\$HOME/.profile"<br><br>\[ -f "\$HOME/.zshrc" \] && PROFILE="\$HOME/.zshrc"<br><br>\[ -f "\$HOME/.bashrc" \] && PROFILE="\$HOME/.bashrc"<br><br>echo "export PATH="\$INSTALL_DIR:\$PATH"" >> "\$PROFILE"<br><br>export PATH="\$INSTALL_DIR:\$PATH"<br><br>\# Verify install<br><br>lumina --version<br><br>echo "Lumina installed successfully."<br><br>echo "Run: lumina run your-program.lum" |

# **42.3 What Must Never Happen**

**IMPORTANT Installer Hard Rules**

NEVER require sudo or admin privileges for the install.

NEVER modify system directories -- install to ~/.lumina/bin only.

NEVER leave partial installs -- clean up on failure.

NEVER break existing PATH -- append only, never replace.

NEVER download without checksum verification.

NEVER print confusing output -- one line per step, clear language.

NEVER require manual PATH restart -- source the profile at the end.

NEVER fail silently -- every error must print a clear message and exit.

**Chapter 43**

**Package Manager Support**

_brew install lumina -- engineers trust their own tools_

The curl installer is fast. Package managers are trusted. Engineers who have been burned by curl scripts before will reach for Homebrew or APT instead. Package manager support also makes Lumina discoverable -- engineers browse available packages and find things they did not know existed.

# **43.1 Priority Order**

Not all package managers are equal in effort and audience. Build in this order:

| **Priority** | **Package Manager**    | **Audience**                                        | **Effort**                             |
| ------------ | ---------------------- | --------------------------------------------------- | -------------------------------------- |
| 1            | Homebrew (macOS)       | macOS developers -- largest early adopter group     | Low -- single Formula.rb file          |
| 2            | Direct binary download | All platforms -- corporate environments, air-gapped | Zero -- GitHub releases already exist  |
| 3            | APT (Ubuntu/Debian)    | Linux server engineers -- data center audience      | Medium -- PPA setup required           |
| 4            | Winget (Windows)       | Windows developers                                  | Medium -- Microsoft submission process |
| 5            | Snap (Linux)           | Alternative Linux distribution                      | Low -- after APT                       |

**NOTE Realistic Timeline Note**

Homebrew and direct binaries ship with v1.8.

APT requires PPA setup and Ubuntu packaging -- may slip to v1.8.

Winget requires Microsoft review process -- timeline uncertain.

Do not block v1.8 launch on APT or Winget.

# **43.2 Homebrew Formula**

| **Formula/lumina.rb -- the Homebrew formula**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| class Lumina < Formula<br><br>desc "Declarative reactive language for IoT and infrastructure monitoring"<br><br>homepage "<https://lumina-lang.dev>"<br><br>version "1.8.0"<br><br>on_macos do<br><br>if Hardware::CPU.arm?<br><br>url "<https://github.com/luminalang/lumina/releases/download/v1.8.0/lumina-macos-arm64.tar.gz>"<br><br>sha256 "REPLACE_WITH_ACTUAL_SHA256"<br><br>else<br><br>url "<https://github.com/luminalang/lumina/releases/download/v1.8.0/lumina-macos-x64.tar.gz>"<br><br>sha256 "REPLACE_WITH_ACTUAL_SHA256"<br><br>end<br><br>end<br><br>def install<br><br>bin.install "lumina"<br><br>bin.install "lumina-lsp"<br><br>end<br><br>test do<br><br>system "#{bin}/lumina", "--version"<br><br>end<br><br>end |

# **43.3 Direct Binary Downloads**

GitHub releases page provides prebuilt binaries for every platform. Engineers who cannot or will not use package managers download the binary directly and place it on their PATH.

| **Binary**             | **Platform**                             |
| ---------------------- | ---------------------------------------- |
| lumina-linux-x64       | Linux x86_64                             |
| lumina-linux-arm64     | Linux ARM64 (Raspberry Pi, AWS Graviton) |
| lumina-macos-arm64     | macOS Apple Silicon                      |
| lumina-macos-x64       | macOS Intel                              |
| lumina-windows-x64.exe | Windows x86_64                           |

Each binary ships alongside a .sha256 checksum file. Engineers verify before running. This is especially important for the data center and infrastructure audience who have strict security requirements.

**Chapter 44**

**VS Code Marketplace**

_Search "Lumina" in VS Code -- install in one click -- LSP works immediately_

The VS Code extension is currently installed manually from a .vsix file. v1.8 puts it on the official marketplace. This is one of the most important distribution channels for Lumina because many engineers discover languages through their editor, not through websites or package managers.

# **44.1 Why the Marketplace Matters**

An engineer opens VS Code. They start exploring a new domain -- IoT monitoring, infrastructure automation. They search the extensions marketplace for tools. If Lumina is there, they find it. If it is not, Lumina does not exist to them.

The marketplace also signals legitimacy. A language with a marketplace extension is a language someone is maintaining. A .vsix file is something a stranger sent you.

# **44.2 What the Extension Provides**

| **Capability**                                 | **Source**                        |
| ---------------------------------------------- | --------------------------------- |
| Syntax highlighting for .lum files             | Grammar definition (v1.4)         |
| Code snippets -- entity, rule, aggregate, etc. | Snippets file (v1.4)              |
| Real-time error squiggles                      | lumina-lsp diagnostics (v1.5)     |
| Hover tooltips -- field type and @doc text     | lumina-lsp hover (v1.5)           |
| Go-to-definition                               | lumina-lsp definition (v1.5)      |
| Document symbols -- outline panel              | lumina-lsp symbols (v1.5)         |
| Completion -- entity and field names           | lumina-lsp completion (v1.5)      |
| Rename symbol                                  | lumina-lsp rename (v1.6)          |
| Find all references                            | lumina-lsp references (v1.6)      |
| Code actions -- quick fixes                    | lumina-lsp code actions (v1.6)    |
| Semantic tokens -- richer highlighting         | lumina-lsp semantic tokens (v1.6) |
| Inlay hints -- field types inline              | lumina-lsp inlay hints (v1.6)     |

# **44.3 Marketplace Metadata**

| **package.json -- marketplace fields**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| {<br><br>"name": "lumina-lang",<br><br>"displayName": "Lumina",<br><br>"description": "Declarative reactive language for IoT and infrastructure monitoring",<br><br>"version": "1.8.0",<br><br>"publisher": "luminalang",<br><br>"categories": \["Programming Languages", "Linters"\],<br><br>"keywords": \["lumina", "reactive", "IoT", "monitoring", "declarative"\],<br><br>"icon": "images/lumina-icon.png",<br><br>"homepage": "<https://lumina-lang.dev>",<br><br>"repository": { "type": "git", "url": "<https://github.com/luminalang/lumina>" },<br><br>"engines": { "vscode": "^1.75.0" },<br><br>"activationEvents": \["onLanguage:lumina"\],<br><br>"contributes": {<br><br>"languages": \[{ "id": "lumina", "extensions": \[".lum"\], "aliases": \["Lumina"\] }\]<br><br>}<br><br>} |

# **44.4 Publication Process**

| **Step**                      | **Action**                                               |
| ----------------------------- | -------------------------------------------------------- |
| 1\. Create publisher account  | Register at marketplace.visualstudio.com as "luminalang" |
| 2\. Get Personal Access Token | Azure DevOps PAT with Marketplace publish scope          |
| 3\. Install vsce              | npm install -g @vscode/vsce                              |
| 4\. Package extension         | vsce package -- produces lumina-lang-1.8.0.vsix          |
| 5\. Publish                   | vsce publish -- submits to marketplace                   |
| 6\. Verify                    | Search "Lumina" in VS Code -- confirm it appears         |
| 7\. Automate                  | GitHub Action to publish on every tagged release         |

**Chapter 45**

**Getting Started Guides**

_One flagship guide. Three secondary guides. Ten minutes to first real alert._

Four domain guides exist but they are not equal. One guide is the face of Lumina -- the first thing engineers read. It must be perfect. The flagship is the delivery fleet. Every other domain guide is secondary. Engineers who have a different domain will find their guide. But the delivery fleet is what Lumina looks like to the world.

**CORE Why Delivery Fleet is the Flagship**

Everyone understands it immediately -- no domain expertise required.

Battery draining. Vehicle going offline. Alert firing. Universally relatable.

A data center example is technically impressive but requires infrastructure knowledge.

An engineer who has never written IoT code understands a drone with a dying battery.

The delivery fleet is the domain where Lumina sells itself in the fewest words.

# **45.1 The Flagship Guide -- Delivery Fleet Monitoring**

Guide structure: what problem we are solving, the complete Lumina program with line-by-line explanation, how to run it, what to expect, three common errors and fixes, what to build next.

| **The complete flagship delivery fleet program**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| \-- A fleet of delivery motos. Each has a battery.<br><br>\-- When battery drops below 20% -- alert the dispatcher.<br><br>\-- When any moto goes offline -- alert immediately.<br><br>\-- When the whole fleet is offline -- critical alert.<br><br>entity Moto {<br><br>battery: Number<br><br>isOnline: Boolean<br><br>label: Text<br><br>isLowBattery := battery < 20<br><br>isCritical := battery < 5<br><br>}<br><br>aggregate FleetStatus over Moto {<br><br>avgBattery := avg(battery)<br><br>onlineCount := count(isOnline)<br><br>anyLow := any(isLowBattery)<br><br>allOffline := all(isOnline)<br><br>}<br><br>rule LowBattery for (m: Moto)<br><br>when m.isLowBattery becomes true<br><br>cooldown 10m {<br><br>alert severity: "warning",<br><br>source: m.label,<br><br>message: "low battery: {m.battery}% on {m.label}"<br><br>} on clear {<br><br>alert severity: "resolved",<br><br>source: m.label,<br><br>message: "battery recovered: {m.battery}%"<br><br>}<br><br>rule MotoOffline for (m: Moto)<br><br>when m.isOnline becomes false {<br><br>alert severity: "critical",<br><br>source: m.label,<br><br>message: "moto offline: {m.label}"<br><br>} on clear {<br><br>alert severity: "resolved",<br><br>source: m.label,<br><br>message: "moto back online: {m.label}"<br><br>}<br><br>rule FleetOffline<br><br>when all Moto.isOnline becomes false {<br><br>alert severity: "critical",<br><br>source: "fleet",<br><br>message: "entire fleet offline -- fleet avg battery: {FleetStatus.avgBattery}%"<br><br>}<br><br>let moto1 = Moto { battery: 80, isOnline: true, label: "moto-north-1" }<br><br>let moto2 = Moto { battery: 45, isOnline: true, label: "moto-north-2" }<br><br>let moto3 = Moto { battery: 12, isOnline: true, label: "moto-south-1" } |

# **45.2 The Three Secondary Guides**

| **Guide** | **Domain**                     | **What it demonstrates**                                          |
| --------- | ------------------------------ | ----------------------------------------------------------------- |
| Guide 2   | Temperature sensor network     | External entities, sync on, prev() for drift detection            |
| Guide 3   | Data center basic monitoring   | ref relationships, multi-condition triggers, aggregate health     |
| Guide 4   | Smart agriculture soil sensors | Frequency conditions, Timestamp type, write action for irrigation |

# **45.3 Guide Quality Standards**

**RULE Every Guide Must Meet These Standards**

Complete in under 10 minutes for a developer who has never seen Lumina.

The complete program is shown first -- no partial snippets.

Every line of the program is explained in plain English.

The guide ends with a real alert firing -- not just "it should work."

Three common errors are listed with exact error codes and exact fixes.

A "what to build next" section points to the next guide or playground.

No assumed knowledge beyond basic programming concepts.

**Chapter 46**

**Error Message Review**

_Every error from L001 to L042 reviewed -- errors that teach instead of confuse_

Error messages are the most underrated part of a language. They are the first thing a confused engineer reads. If the message teaches -- the engineer fixes the problem and continues. If it confuses -- the engineer gives up and uninstalls. v1.8 reviews every error code and rewrites any that does not meet the teaching standard.

**CORE The Error Message Standard**

Every error message must answer three questions:

1\. What went wrong -- in plain English, no jargon.

2\. Why Lumina does not allow this -- one sentence explanation.

3\. How to fix it -- a concrete suggestion or example.

An error that does not answer all three questions fails the standard.

# **46.1 Before and After -- The Standard Applied**

| **L024 -- Before review (current)** |
| ----------------------------------- |
| L024: invalid prev() usage          |

| **L024 -- After review (v1.8 standard)**                                                                                                                                                                                                                                                                                                            |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| L024: prev() cannot be applied to a derived field.<br><br>batteryDrop := prev(batteryDrop) - batteryDrop -- ERROR here<br><br>^^^^^^^^^^^<br><br>"batteryDrop" is a derived field. It has no stored previous value.<br><br>prev() only works on fields declared with a type (Number, Boolean, Text).<br><br>Did you mean: prev(battery) - battery ? |

# **46.2 Complete Error Code Review**

| **Code**  | **Category**     | **Review focus**                                       |
| --------- | ---------------- | ------------------------------------------------------ |
| L001      | Unknown entity   | Suggest closest match by name similarity               |
| L002      | Unknown field    | Show available fields on the entity                    |
| L003      | Type mismatch    | Show expected type and actual type clearly             |
| L004      | Invalid action   | Explain which actions are valid in this context        |
| L005      | Unknown instance | Suggest declaring the instance first                   |
| L006-L010 | Parser errors    | Rewrite to show exactly what was expected vs found     |
| L011-L015 | Analyzer errors  | Add concrete fix examples for each                     |
| L016-L023 | v1.4 errors      | Review fn and import related messages                  |
| L024-L034 | v1.5 errors      | prev/alert/aggregate/cooldown messages -- add examples |
| L035-L042 | v1.6 errors      | ref/compound/frequency/write/Timestamp -- all new      |

# **46.3 The Error Reference Page**

Every error code gets a dedicated section on the documentation website. Searchable by code number. Each section shows: the error message, what triggers it, a minimal bad example, a corrected good example, and why Lumina enforces this rule.

Engineers who hit an error should be able to search the code, find the page in under 10 seconds, and understand the fix in under 30 seconds. That is the standard.

**Chapter 47**

**Docker Image**

_docker run luminalang/runtime -- for engineers who do not install runtimes_

Infrastructure and data center engineers do not install language runtimes on production machines. They run containers. The Docker image makes Lumina usable in those environments without any local installation. It is also the path for CI/CD pipelines that want to validate Lumina programs as part of a build.

# **47.1 The Docker Image**

| **Dockerfile -- the Lumina runtime image**                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| FROM debian:bookworm-slim<br><br>RUN apt-get update && apt-get install -y \\<br><br>ca-certificates \\<br><br>&& rm -rf /var/lib/apt/lists/\*<br><br>COPY lumina /usr/local/bin/lumina<br><br>COPY lumina-lsp /usr/local/bin/lumina-lsp<br><br>RUN chmod +x /usr/local/bin/lumina /usr/local/bin/lumina-lsp<br><br>WORKDIR /lumina<br><br>ENTRYPOINT \["lumina"\]<br><br>CMD \["--help"\] |

# **47.2 Usage Patterns**

| **Running a Lumina program in Docker**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| \-- Run a single program<br><br>docker run -v \$(pwd):/lumina luminalang/runtime run fleet.lum<br><br>\-- Check a program for errors without running<br><br>docker run -v \$(pwd):/lumina luminalang/runtime check fleet.lum<br><br>\-- Docker Compose with MQTT broker<br><br>\# docker-compose.yml<br><br>services:<br><br>lumina:<br><br>image: luminalang/runtime<br><br>volumes:<br><br>\- ./programs:/lumina<br><br>command: run datacenter.lum<br><br>depends_on:<br><br>\- mqtt<br><br>mqtt:<br><br>image: eclipse-mosquitto<br><br>ports:<br><br>\- "1883:1883" |

# **47.3 Image Variants**

| **Image tag**             | **Contents**                             | **Use case**                 |
| ------------------------- | ---------------------------------------- | ---------------------------- |
| luminalang/runtime:latest | lumina + lumina-lsp, minimal Debian base | Production deployments       |
| luminalang/runtime:alpine | lumina + lumina-lsp, Alpine Linux base   | Smallest possible image size |
| luminalang/runtime:dev    | runtime + development tools + shell      | Development and debugging    |

**NOTE Docker Priority Note**

Docker is important for the infrastructure audience but not the primary onboarding path.

Most engineers will try the playground first, then install locally.

Docker matters most for: production deployments, CI/CD, and corporate environments.

Build and publish the image but do not over-prioritize vs the installer and playground.

**Chapter 48**

**Playground Polish**

_The real entry point -- the playground is where engineers first feel Lumina_

The playground is not a secondary feature. It is the primary entry point for most engineers who discover Lumina. They will not install anything before trying it. The playground must be excellent -- fast, responsive, alive on arrival, and smooth enough that the first experience is the one that converts. v1.8 polishes every rough edge.

**CORE The Playground Is the Killer Demo**

ChatGPT suggested building a separate killer demo system. That is wrong.

The playground already IS the killer demo.

Making the playground excellent is the right instinct.

Building a separate demo system adds complexity without adding value.

Every hour spent on a separate demo is an hour not spent polishing the playground.

# **48.1 What Changes in v1.8**

| **Area**              | **Change**                                                                        |
| --------------------- | --------------------------------------------------------------------------------- |
| Load time             | WASM binary size reduction -- target under 500KB, load under 2 seconds            |
| Pre-loaded example    | Delivery fleet program pre-loaded on arrival -- already running, not blank editor |
| Above the fold        | Embedded in website landing page -- visible without scrolling                     |
| Mobile responsive     | Engineers check things on phones -- playground must work on mobile                |
| Error display         | Error messages shown inline with L-codes and fix suggestions                      |
| Share URL reliability | Test across Chrome, Firefox, Safari, Edge -- URL encoding bulletproof             |
| Alert timeline polish | Color coding sharper, timestamps cleaner, resolved events distinct                |
| State panel polish    | Derived field highlighting cleaner, alert badges more visible                     |
| Virtual clock UX      | Speed selector clearer, pause/resume more obvious                                 |
| Example selector      | Dropdown to switch between domain examples without page reload                    |

# **48.2 The Pre-Loaded Experience**

When an engineer visits lumina-lang.dev or opens the playground directly, they see the delivery fleet program already loaded and already running. Three moto instances visible in the state panel. The virtual clock ticking at 1x. One alert already in the timeline from moto3 whose battery is at 12%.

The engineer did not write anything. They did not click run. Lumina is already reacting. Then they drag the battery slider on moto2 below 20% and watch a warning alert fire instantly. That moment -- that first reaction -- is when they understand what Lumina is.

| **The pre-loaded playground state**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \-- Already loaded when engineer arrives:<br><br>let moto1 = Moto { battery: 80, isOnline: true, label: "moto-north-1" }<br><br>let moto2 = Moto { battery: 45, isOnline: true, label: "moto-north-2" }<br><br>let moto3 = Moto { battery: 12, isOnline: true, label: "moto-south-1" }<br><br>\-- Alert timeline already shows:<br><br>\-- \[WARNING\] moto-south-1 -- low battery: 12% (fired on load)<br><br>\-- State panel shows three instance cards.<br><br>\-- moto3 has a red ALERT badge.<br><br>\-- FleetStatus aggregate shows avgBattery: 45.7<br><br>\-- Engineer drags moto2 battery below 20.<br><br>\-- New warning fires instantly.<br><br>\-- avgBattery recomputes.<br><br>\-- Engineer understands Lumina in 30 seconds. |

# **48.3 Performance Targets**

| **Metric**              | **Target**                              |
| ----------------------- | --------------------------------------- |
| WASM binary size        | Under 500KB gzipped                     |
| Initial load time       | Under 2 seconds on a 10Mbps connection  |
| Rule evaluation latency | Under 16ms per slider drag (60fps feel) |
| Share URL generation    | Instant -- under 50ms                   |
| Share URL load          | Pre-populated editor in under 1 second  |
| Mobile layout           | Usable on 375px wide screen (iPhone SE) |

# **48.4 The Example Selector**

A dropdown in the playground header lets engineers switch between pre-built domain examples without losing the share URL feature. Switching loads a fresh program and resets the state panel and alert timeline.

| **Example**              | **Pre-loaded state**                             |
| ------------------------ | ------------------------------------------------ |
| Delivery Fleet (default) | 3 motos, one low battery alert already fired     |
| Temperature Sensors      | 3 sensors, one showing a spike in prev() delta   |
| Data Center              | 2 servers with ref to cooling units, one at-risk |
| Smart Agriculture        | 2 soil sensors, one showing dry condition        |

**Appendix**

**v1.8 Chapter Summary**

_Eight chapters -- no new language features -- the experience that makes Lumina ready_

| **Chapter**                  | **What it delivers**                                               |
| ---------------------------- | ------------------------------------------------------------------ |
| 41 -- Website                | lumina-lang.dev live with playground embedded above the fold       |
| 42 -- One-line installer     | curl install that just works on macOS and Linux                    |
| 43 -- Package managers       | Homebrew and direct binaries at launch, APT and Winget to follow   |
| 44 -- VS Code Marketplace    | Search Lumina in VS Code -- install in one click                   |
| 45 -- Getting Started Guides | Flagship delivery fleet guide plus three domain guides             |
| 46 -- Error Message Review   | Every L001-L042 rewritten to teach not just report                 |
| 47 -- Docker Image           | luminalang/runtime on Docker Hub for infrastructure users          |
| 48 -- Playground Polish      | Pre-loaded example, WASM performance, mobile layout, error display |

# **What Comes After v1.8**

| **Version** | **Identity**                                                                               |
| ----------- | ------------------------------------------------------------------------------------------ |
| v1.8        | Validation -- fix what real usage reveals after controlled early access                    |
| v1.9        | Performance and stability -- conditional on v1.8 findings                                  |
| v2          | Scale + first public users -- namespaces, rule templates, Lumina State Store design begins |

**YES v1.8 Definition of Done**

lumina-lang.dev is live and loads in under 2 seconds.

Playground is embedded above the fold with fleet example pre-loaded.

curl install works on macOS and Linux without errors.

brew install lumina works on macOS.

VS Code extension is searchable and installable from the marketplace.

Flagship delivery fleet guide completed in under 10 minutes.

All four domain guides published and tested.

Every error code L001-L042 has a clear message with fix suggestion.

Docker image published at luminalang/runtime:latest.

Playground loads under 2 seconds, reacts under 16ms.