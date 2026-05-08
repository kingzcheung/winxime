# 自动 MSI 打包脚本

Write-Host "Building MSI..." -ForegroundColor Cyan

# 添加 WiX 到 PATH
$env:PATH += ";C:\Program Files (x86)\WiX Toolset v3.14\bin"

# 构建 MSI
cargo wix -p winxime-server

# 检查结果
$msi = Get-ChildItem "target\wix\*.msi" -ErrorAction SilentlyContinue

if ($msi) {
    Write-Host ""
    Write-Host "Success: $($msi.Name)" -ForegroundColor Green
    Write-Host "Size: $([math]::Round($msi.Length / 1KB)) KB" -ForegroundColor White
    Write-Host ""
    Write-Host "Install: Right-click MSI -> Install" -ForegroundColor Yellow
} else {
    Write-Host "Failed" -ForegroundColor Red
}