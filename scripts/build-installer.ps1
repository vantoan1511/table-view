param(
    [string]$Version = "0.2.11"
)

# Table-View Installer Build Script
# This script prepares the staging directory and triggers Inno Setup.

$ProjectRoot = Get-Item $PSScriptRoot\..
$DistDir = "$ProjectRoot\dist"
$StagingDir = "$DistDir\staging"
$InstallerDir = "$ProjectRoot\scripts"

Write-Host "--- Preparing Staging Area ---" -ForegroundColor Cyan

# Create staging directory
if (Test-Path $StagingDir) {
    Remove-Item -Recurse -Force $StagingDir
}
New-Item -ItemType Directory -Path $StagingDir | Out-Null
New-Item -ItemType Directory -Path "$StagingDir\extensions" | Out-Null

# Copy main binaries
Write-Host "Copying main binaries..."
Copy-Item "$DistDir\table-view\table-view-win_x64.exe" "$StagingDir\table-view.exe"
Copy-Item "$DistDir\table-view\resources.neu" "$StagingDir\resources.neu"
Copy-Item "$DistDir\table-view\icon.ico" "$StagingDir\icon.ico"

# Copy extension
Write-Host "Copying extensions..."
$ExtensionSource = "$DistDir\table-view\extensions\db-bridge"
$ExtensionDest = "$StagingDir\extensions\db-bridge"
New-Item -ItemType Directory -Path $ExtensionDest | Out-Null

# Only copy the executable and any necessary runtime files
# Excluding: src, target, vendor, Cargo.*
Copy-Item "$ExtensionSource\db-bridge.exe" "$ExtensionDest\db-bridge.exe"

Write-Host "--- Staging Complete ---" -ForegroundColor Green

# Locate ISCC.exe
$ISCC = "ISCC.exe"
if (!(Get-Command $ISCC -ErrorAction SilentlyContinue)) {
    $CommonPaths = @(
        "D:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
    )
    foreach ($path in $CommonPaths) {
        if (Test-Path $path) {
            $ISCC = $path
            break
        }
    }
}

if (!(Get-Command $ISCC -ErrorAction SilentlyContinue) -and !(Test-Path $ISCC)) {
    Write-Error "ISCC.exe (Inno Setup Compiler) not found in PATH or standard locations. Please ensure Inno Setup is installed."
    exit 1
}

Write-Host "--- Compiling Installer (Version: $Version) ---" -ForegroundColor Cyan
& $ISCC /DAppVersion="$Version" "$InstallerDir\installer.iss"

if ($LASTEXITCODE -eq 0) {
    Write-Host "--- Success! Installer created in scripts\Output ---" -ForegroundColor Green
}
else {
    Write-Error "Inno Setup compilation failed."
    exit $LASTEXITCODE
}
