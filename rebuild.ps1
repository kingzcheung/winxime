# Rebuild and test winxime
# Run this script after stopping all winxime processes

Write-Host "Step 1: Unregistering DLL..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/u /s", "target\debug\winxime_tsf.dll"
Write-Host "Waiting 10 seconds for DLL to be released..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

Write-Host "Step 2: Stopping server..." -ForegroundColor Yellow
Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 3

Write-Host "Step 3: Building..." -ForegroundColor Yellow
cargo build --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Step 4: Registering DLL..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/s", "target\debug\winxime_tsf.dll"
Start-Sleep -Seconds 2

Write-Host "Step 5: Starting server..." -ForegroundColor Yellow
Start-Process -FilePath "target\debug\winxime-server.exe"
Start-Sleep -Seconds 3

Write-Host "`nDone! Server is running. Test input in Windows Terminal or Notepad." -ForegroundColor Green