# Build librime for winxime
# Run in PowerShell: ./build-librime.ps1

param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$librimeDir = Join-Path $PSScriptRoot "librime"
$distDir = Join-Path $librimeDir "dist"
$rimeDll = Join-Path $distDir "rime.dll"

if ($rimeDll -and !$Force) {
    Write-Host "librime already built: $rimeDll" -ForegroundColor Green
    Write-Host "Use -Force to rebuild"
    return
}

Write-Host "Building librime..." -ForegroundColor Yellow

# Find Visual Studio
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vsWhere)) {
    Write-Error "vswhere.exe not found. Please install Visual Studio 2022"
    exit 1
}

$vsInstallPath = & $vsWhere -latest -property installationPath
Write-Host "VS installed at: $vsInstallPath"

# Setup VS environment
$vcvarsPath = Join-Path $vsInstallPath "VC\Auxiliary\Build\vcvars64.bat"
if (-not (Test-Path $vcvarsPath)) {
    Write-Error "vcvars64.bat not found"
    exit 1
}

# Run build in cmd with VS environment
cmd /c """$vcvarsPath"" && cd /d ""$librimeDir"" && copy env.bat.template env.bat && build.bat deps && build.bat librime"

if (Test-Path $rimeDll) {
    Write-Host "Build complete: $rimeDll" -ForegroundColor Green
} else {
    Write-Error "Build failed. Check the output above."
}