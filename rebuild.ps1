# Rebuild and test winxime (development workflow)

$iconPath = "$PSScriptRoot\resource\icon.ico"
$registerExe = "$PSScriptRoot\target\debug\winxime-tsf-register.exe"

Write-Host "Step 1: Stopping server..." -ForegroundColor Yellow
cargo run -p winxime-server -- /q 2>&1 | Out-Null
Start-Sleep -Seconds 2
Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

Write-Host "Step 2: Unregistering COM DLL..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/u /s", "$PSScriptRoot\target\debug\winxime_tsf.dll" -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

Write-Host "Step 3: Unregistering profile..." -ForegroundColor Yellow
if (Test-Path $registerExe) {
    Start-Process -Verb RunAs -Wait -FilePath $registerExe -ArgumentList "-u"
    Start-Sleep -Seconds 2
}

Write-Host "Step 4: Building..." -ForegroundColor Yellow
cargo build --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Step 5: Registering COM DLL (no profile)..." -ForegroundColor Yellow
# Only register COM class factory, not profile (use winxime-tsf-register for profile)
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/s", "$PSScriptRoot\target\debug\winxime_tsf.dll"
Start-Sleep -Seconds 2

Write-Host "Step 6: Registering profile with icon..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath $registerExe -ArgumentList "-r", $iconPath
Start-Sleep -Seconds 2

Write-Host "Step 7: Enabling..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath $registerExe -ArgumentList "-i"
Start-Sleep -Seconds 1

Write-Host "Step 8: Starting server (debug mode)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cargo run -p winxime-server" -WindowStyle Normal
Start-Sleep -Seconds 3

Write-Host "`nDone!" -ForegroundColor Green
Write-Host "Server running in separate window (with console log)" -ForegroundColor Cyan
Write-Host "Test input in Notepad or any application" -ForegroundColor White