$ErrorActionPreference = "Stop"

$LUMINA_HOME = if ($env:LUMINA_HOME) { $env:LUMINA_HOME } else { Join-Path $HOME ".lumina" }
$BIN_DIR = Join-Path $LUMINA_HOME "bin"
$BASE_URL = "https://woijupkxzzakmkneyxwk.supabase.co/storage/v1/object/public/Lumina"

# --- Output Formatting ---
function Write-Info { param([string]$msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success { param([string]$msg) Write-Host "[SUCCESS] $msg" -ForegroundColor Green }
function Write-ErrorMsg { param([string]$msg) Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }

# --- Architecture Detection ---
$Arch = (Get-WmiObject -Class Win32_Processor).Architecture
if ($Arch -eq 9) {
    $Platform = "windows-x64"
} elseif ($Arch -eq 12) {
    $Platform = "windows-arm64"
    Write-ErrorMsg "ARM64 Windows is not officially supported yet. Please build from source."
} else {
    Write-ErrorMsg "Unsupported platform architecture ($Arch). Please download manually."
}

Write-Info "Installing Lumina v2.1.0 (Architect) for $Platform..."

if (-not (Test-Path $BIN_DIR)) {
    New-Item -ItemType Directory -Path $BIN_DIR -Force | Out-Null
}

function Download-Binary {
    param([string]$Name, [string]$UrlSuffix, [string]$BinName)
    $Url = "$BASE_URL/lumina-$UrlSuffix"
    $Dest = Join-Path $BIN_DIR $BinName

    Write-Info "Downloading $BinName..."
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing
    } catch {
        Write-ErrorMsg "Failed to download $BinName from $Url"
    }
}

# The filenames uploaded to Supabase Storage:
# lumina-windows-x64.exe
# lumina-windows-x64-lsp.exe
Download-Binary -Name "core" -UrlSuffix "windows-x64.exe" -BinName "lumina.exe"
Download-Binary -Name "lsp" -UrlSuffix "windows-x64-lsp.exe" -BinName "lumina-lsp.exe"

# --- Path Injection ---
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$BIN_DIR*") {
    Write-Info "Adding Lumina to PATH"
    $NewPath = "$BIN_DIR;$UserPath"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    $env:PATH = "$BIN_DIR;$env:PATH"
} else {
    Write-Info "Lumina is already in your PATH."
}

# --- Verify & Setup ---
Write-Success "Lumina successfully installed!"

$LuminaExe = Join-Path $BIN_DIR "lumina.exe"
if (Test-Path $LuminaExe) {
    & $LuminaExe setup
    Write-Host ""
    Write-Success "Setup complete! Start coding:"
    Write-Host "  Run:   lumina run your-program.lum"
    Write-Host "  Check: lumina check your-program.lum"
    Write-Host "  Docs:  https://lumina-lang.web.app/docs"
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host "  Please RESTART your terminal to use Lumina,"
    Write-Host "  or run this to update your current session's PATH:"
    Write-Host ""
    Write-Host "    `$env:PATH = `"$BIN_DIR;`$env:PATH`"" -ForegroundColor Green
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}
