# 完整卸载 MSI 安装的 Xime

Write-Host "Uninstalling Xime..." -ForegroundColor Yellow

# 1. 停止服务器
Write-Host "Step 1: Stopping server..." -ForegroundColor Yellow
Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# 2. 卸载所有 MSI 版本
Write-Host "Step 2: Uninstalling MSI..." -ForegroundColor Yellow
$products = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like '*winxime*' -or $_.Name -like '*xime*' }
foreach ($product in $products) {
    Write-Host "  Removing $($product.Name)..." -ForegroundColor Gray
    msiexec /x $product.IdentifyingNumber /qn
    Start-Sleep -Seconds 2
}

# 3. 清理残留文件
Write-Host "Step 3: Cleaning residual files..." -ForegroundColor Yellow
Remove-Item "C:\Program Files\winxime-server" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Windows\System32\winxime_tsf.dll" -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Windows\Installer\icon.ico" -Force -ErrorAction SilentlyContinue

# 4. 清理注册表
Write-Host "Step 4: Cleaning registry..." -ForegroundColor Yellow
reg delete "HKLM\SOFTWARE\Classes\CLSID\{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}" /f 2>$null
reg delete "HKLM\SOFTWARE\Microsoft\CTF\TIP\{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}" /f 2>$null
reg delete "HKLM\SOFTWARE\Classes\AppID\{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}" /f 2>$null
reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v XimeServer /f 2>$null
reg delete "HKCU\SOFTWARE\Microsoft\CTF\TIP\{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}" /f 2>$null

# 5. 验证清理
Write-Host ""
Write-Host "Verification:" -ForegroundColor Cyan
$remaining = Get-WmiObject -Class Win32_Product -ErrorAction SilentlyContinue | Where-Object { $_.Name -like '*winxime*' }
if ($remaining) {
    Write-Host "  Warning: Some MSI products still registered" -ForegroundColor Yellow
} else {
    Write-Host "  MSI products: None (OK)" -ForegroundColor Green
}

if (Test-Path "C:\Program Files\winxime-server") {
    Write-Host "  Warning: Folder still exists" -ForegroundColor Yellow
} else {
    Write-Host "  Installation folder: Removed (OK)" -ForegroundColor Green
}

if (Test-Path "C:\Windows\System32\winxime_tsf.dll") {
    Write-Host "  Warning: DLL still in System32" -ForegroundColor Yellow
} else {
    Write-Host "  System DLL: Removed (OK)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Green