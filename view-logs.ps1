# View winxime logs

Write-Host "=== TSF DLL Log (last 40 lines) ===" -ForegroundColor Cyan
Get-Content "$env:TEMP\winxime_tsf.log" -Tail 40 -ErrorAction SilentlyContinue

Write-Host "`n=== Server Status ===" -ForegroundColor Cyan
Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue | Format-Table Id, StartTime, CPU -AutoSize

Write-Host "`nPress any key to exit..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")