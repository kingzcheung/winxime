# MSI 打包脚本

param(
    [string]$Version = "0.1.0"
)

Write-Host "Building Xime MSI v$Version..." -ForegroundColor Cyan

# 添加 WiX 到 PATH
$env:PATH += ";C:\Program Files (x86)\WiX Toolset v3.14\bin"

# 1. 构建 release 版本
Write-Host "Step 1: Building release..." -ForegroundColor Yellow
cargo build --release --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# 2. 复制 data 文件
Write-Host "Step 2: Copying data files..." -ForegroundColor Yellow
if (Test-Path "target\release\data") {
    Remove-Item "target\release\data" -Recurse -Force
}
Copy-Item "librime\data" "target\release\data" -Recurse

# 3. 用 heat 收集 data 文件
Write-Host "Step 3: Harvesting data files..." -ForegroundColor Yellow
heat dir "target\release\data" -o "crates\winxime-server\wix\data.wxs" -dr DataFolder -cg DataFiles -var var.DataDir -sreg -srd -ag

# 4. 编译和链接 MSI
Write-Host "Step 4: Building MSI..." -ForegroundColor Yellow
candle -arch x64 "crates\winxime-server\wix\main.wxs" "crates\winxime-server\wix\data.wxs" -ext WixUIExtension -ext WixUtilExtension "-dCargoTargetBinDir=target\release" "-dDataDir=target\release\data" "-dVersion=$Version" -out "target\wix\"
light "target\wix\main.wixobj" "target\wix\data.wixobj" -ext WixUIExtension -ext WixUtilExtension -out "target\wix\xime-$Version.msi"

# 5. 检查结果
$msi = Get-Item "target\wix\xime-$Version.msi" -ErrorAction SilentlyContinue
if ($msi) {
    Write-Host ""
    Write-Host "Success: $($msi.FullName)" -ForegroundColor Green
    Write-Host "Size: $([math]::Round($msi.Length / 1MB, 2)) MB" -ForegroundColor White
    Write-Host ""
    Write-Host "Install: msiexec /i $($msi.FullName)" -ForegroundColor Yellow
} else {
    Write-Host "MSI build failed!" -ForegroundColor Red
    exit 1
}