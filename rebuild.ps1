# Rebuild and test winxime (development workflow)

Write-Host "Step 1: Unregistering DLL..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/u /s", "target\debug\winxime_tsf.dll"
Start-Sleep -Seconds 2

Write-Host "Step 2: Stopping server..." -ForegroundColor Yellow
cargo run -p winxime-server -- /q 2>&1 | Out-Null
Start-Sleep -Seconds 2
Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

Write-Host "Step 3: Building..." -ForegroundColor Yellow
cargo build --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Step 4: Registering DLL..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/s", "target\debug\winxime_tsf.dll"
Start-Sleep -Seconds 2

Write-Host "Step 5: Starting server (debug mode)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cargo run -p winxime-server" -WindowStyle Normal
Start-Sleep -Seconds 3

Write-Host "`nDone!" -ForegroundColor Green
Write-Host "Server running in separate window (with console log)" -ForegroundColor Cyan
Write-Host "Test input in Notepad or any application" -ForegroundColor White