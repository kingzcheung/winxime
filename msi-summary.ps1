# MSI 打包完成总结

Write-Host "MSI Package Summary" -ForegroundColor Cyan
Write-Host ""

$msi = Get-Item "target\wix\*.msi"
Write-Host "File: $($msi.Name)" -ForegroundColor Green
Write-Host "Size: $([math]::Round($msi.Length / 1MB, 2)) MB" -ForegroundColor White
Write-Host "Created: $($msi.LastWriteTime)" -ForegroundColor White

Write-Host ""
Write-Host "Included Files:" -ForegroundColor Yellow
Write-Host "  - winxime-server.exe (main executable)" -ForegroundColor White
Write-Host "  - winxime-setup.exe (settings app)" -ForegroundColor White
Write-Host "  - winxime_tsf.dll (TSF DLL)" -ForegroundColor White
Write-Host "  - rime.dll (Rime library)" -ForegroundColor White
Write-Host "  - data/ (Rime schemas & dictionaries)" -ForegroundColor White

Write-Host ""
Write-Host "To Install:" -ForegroundColor Yellow
Write-Host "  Right-click MSI -> Install" -ForegroundColor White
Write-Host "  Or: msiexec /i $($msi.FullName)" -ForegroundColor White