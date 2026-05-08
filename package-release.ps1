# Build release version and prepare distribution package

Write-Host "Building release version..." -ForegroundColor Yellow

# 1. Build all crates in release mode
cargo build --release --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Build complete" -ForegroundColor Green

# 2. Create distribution directory
$distDir = "dist"
if (Test-Path $distDir) {
    Remove-Item $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir | Out-Null

Write-Host "Copying files to $distDir..." -ForegroundColor Yellow

# 3. Copy executables and DLL
Copy-Item "target\release\winxime-server.exe" $distDir
Copy-Item "target\release\winxime_tsf.dll" $distDir
Copy-Item "target\release\winxime-setup.exe" $distDir -ErrorAction SilentlyContinue
Copy-Item "target\release\winxime-tsf-register.exe" $distDir
Copy-Item "resource\trayicon\zh_light.ico" "$distDir\icon.ico"

# 4. Copy rime.dll from debug (or build release version)
if (Test-Path "target\debug\rime.dll") {
    Copy-Item "target\debug\rime.dll" $distDir
}

# 5. Copy data files
$dataDir = "$distDir\data"
New-Item -ItemType Directory -Path $dataDir | Out-Null

Copy-Item "librime\data\*" $dataDir -Recurse

# 6. Create install script
$installScript = @"
@echo off
echo Installing Xime Input Method...
echo.
echo Step 1: Registering input method...
"%~dp0winxime-tsf-register.exe" -r "%~dp0icon.ico"
echo.
echo Step 2: Enabling input method...
"%~dp0winxime-tsf-register.exe" -i
echo.
echo Step 3: Starting server...
start "" "%~dp0winxime-server.exe"
echo.
echo Installation complete!
echo Please add input method in Windows Settings if not auto-enabled.
pause
"@

Set-Content -Path "$distDir\install.bat" -Value $installScript

# 7. Create uninstall script
$uninstallScript = @"
@echo off
echo Uninstalling Xime Input Method...
echo.
echo Stopping server...
"%~dp0winxime-tsf-register.exe" -s
timeout /t 2 /nobreak >nul
echo.
echo Unregistering input method...
"%~dp0winxime-tsf-register.exe" -u
echo.
echo Uninstallation complete!
echo You can now delete this folder.
pause
"@

Set-Content -Path "$distDir\uninstall.bat" -Value $uninstallScript

# 8. Create README
$readme = @"
Xime Wubi Input Method - Windows Version

Files:
  winxime-server.exe        - Backend server (candidate window + Rime engine)
  winxime_tsf.dll           - TSF input framework DLL
  winxime-tsf-register.exe  - TSF registration tool
  winxime-setup.exe         - Settings application (optional)
  rime.dll                  - Rime engine library
  data/                     - Rime data files (schemas, dictionaries)

Installation:
  1. Run install.bat (requires administrator privileges)
  2. Add input method in Windows Settings
  3. Start server (auto-starts with install.bat)

Usage:
  - Switch input method: Win + Space
  - Type Chinese using Wubi encoding
  - Candidate window shows below cursor

Uninstallation:
  - Run uninstall.bat (requires administrator privileges)

For development:
  - Debug build: cargo build
  - Release build: cargo build --release
"@

Set-Content -Path "$distDir\README.txt" -Value $readme

# 9. List files
Write-Host "`nDistribution contents:" -ForegroundColor Cyan
Get-ChildItem $distDir | ForEach-Object {
    Write-Host "  $_" -ForegroundColor White
}

# 10. Create zip (optional)
$zipFile = "xime-release.zip"
if (Get-Command Compress-Archive -ErrorAction SilentlyContinue) {
    Write-Host "`nCreating zip package..." -ForegroundColor Yellow
    Compress-Archive -Path "$distDir\*" -DestinationPath $zipFile -Force
    Write-Host "Created: $zipFile" -ForegroundColor Green
    Write-Host "Deleting $zipFile..." -ForegroundColor Yellow
    Remove-Item $zipFile -Force
}

Write-Host "`nDone! Distribution ready in: $distDir" -ForegroundColor Green
Write-Host "To install: Run $distDir\install.bat (as administrator)" -ForegroundColor Cyan