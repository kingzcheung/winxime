$msiPath = "target\wix\xime-0.1.0.msi"

if (-not (Test-Path $msiPath)) {
    Write-Host "MSI not found: $msiPath" -ForegroundColor Red
    Write-Host "Run msi-build.ps1 first" -ForegroundColor Yellow
    exit 1
}

Write-Host "Uninstalling old version..." -ForegroundColor Yellow
Start-Process msiexec -ArgumentList "/x `{C82D0C18-FE61-4F32-BB15-1D87BC5912A2`} /qn /norestart" -Wait -NoNewWindow
Write-Host "Old version uninstalled (or not present)" -ForegroundColor Green

Write-Host "Installing new version..." -ForegroundColor Yellow
Start-Process msiexec -ArgumentList "/i `"$msiPath`" /passive" -Wait -NoNewWindow

Write-Host "Installation complete!" -ForegroundColor Green
Write-Host "Location: C:\Program Files\Xime" -ForegroundColor Cyan