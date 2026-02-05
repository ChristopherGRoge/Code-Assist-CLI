param(
    [Parameter(Position=0)]
    [ValidatePattern('^(stable|latest|\d+\.\d+\.\d+(-[^\s]+)?)$')]
    [string]$Target = "latest"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# Check for 32-bit Windows
if (-not [Environment]::Is64BitProcess) {
    Write-Error "Claude Code does not support 32-bit Windows. Please use a 64-bit version of Windows."
    exit 1
}

$GCS_BUCKET = "https://storage.googleapis.com/claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819/claude-code-releases"
$DOWNLOAD_DIR = "$env:USERPROFILE\.claude\downloads"

# Get script directory for local fallback files
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Definition
$LOCAL_DIR = Join-Path $SCRIPT_DIR "local"
$UseLocalFallback = $false

# Always use x64 for Windows (ARM64 Windows can run x64 through emulation)
$platform = "win32-x64"
New-Item -ItemType Directory -Force -Path $DOWNLOAD_DIR | Out-Null

# Always download latest version (which has the most up-to-date installer)
$version = $null
try {
    $version = Invoke-RestMethod -Uri "$GCS_BUCKET/latest" -ErrorAction Stop
}
catch {
    # Try local fallback
    $localLatestPath = Join-Path $LOCAL_DIR "latest"
    if (Test-Path $localLatestPath) {
        Write-Output "Remote download failed, using local fallback..."
        $version = Get-Content $localLatestPath -Raw
        $version = $version.Trim()
        $UseLocalFallback = $true
    }
    else {
        Write-Error "Failed to get latest version (remote unreachable and no local fallback): $_"
        exit 1
    }
}

$manifest = $null
try {
    $manifest = Invoke-RestMethod -Uri "$GCS_BUCKET/$version/manifest.json" -ErrorAction Stop
}
catch {
    # Try local fallback
    $localManifestPath = Join-Path $LOCAL_DIR "$version\manifest.json"
    if (Test-Path $localManifestPath) {
        Write-Output "Remote download failed, using local fallback..."
        $manifest = Get-Content $localManifestPath -Raw | ConvertFrom-Json
        $UseLocalFallback = $true
    }
    else {
        Write-Error "Failed to get manifest (remote unreachable and no local fallback): $_"
        exit 1
    }
}

$checksum = $manifest.platforms.$platform.checksum
if (-not $checksum) {
    Write-Error "Platform $platform not found in manifest"
    exit 1
}

# Download and verify
$binaryPath = "$DOWNLOAD_DIR\claude-$version-$platform.exe"
try {
    Invoke-WebRequest -Uri "$GCS_BUCKET/$version/$platform/claude.exe" -OutFile $binaryPath -ErrorAction Stop
}
catch {
    # Try local fallback
    $localBinaryPath = Join-Path $LOCAL_DIR "$version\$platform\claude.exe"
    if (Test-Path $localBinaryPath) {
        Write-Output "Remote download failed, using local fallback..."
        Copy-Item $localBinaryPath $binaryPath
        $UseLocalFallback = $true
    }
    else {
        Write-Error "Failed to download binary (remote unreachable and no local fallback): $_"
        if (Test-Path $binaryPath) {
            Remove-Item -Force $binaryPath
        }
        exit 1
    }
}

# Calculate checksum
$actualChecksum = (Get-FileHash -Path $binaryPath -Algorithm SHA256).Hash.ToLower()

if ($actualChecksum -ne $checksum) {
    Write-Error "Checksum verification failed"
    Remove-Item -Force $binaryPath
    exit 1
}

# Run claude install to set up launcher and shell integration
Write-Output "Setting up Claude Code..."
try {
    if ($Target) {
        & $binaryPath install $Target
    }
    else {
        & $binaryPath install
    }
}
finally {
    try {
        # Clean up downloaded file
        # Wait a moment for any file handles to be released
        Start-Sleep -Seconds 1
        Remove-Item -Force $binaryPath
    }
    catch {
        Write-Warning "Could not remove temporary file: $binaryPath"
    }
}

Write-Output ""
Write-Output "$([char]0x2705) Installation complete!"
if ($UseLocalFallback) {
    Write-Output "Note: Installation used local fallback files (remote was unreachable)"
}
Write-Output ""
