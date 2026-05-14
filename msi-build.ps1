# MSI build script

param(
    [string]$Version = "0.1.0"
)

Write-Host "Building Xime MSI v$Version..." -ForegroundColor Cyan

# Add WiX v3.14 to PATH
$env:PATH += ";C:\Program Files (x86)\WiX Toolset v3.14\bin"

# 1. Build release
Write-Host "Step 1: Building release..." -ForegroundColor Yellow
$files = Get-ChildItem -Path "crates/winxime-setup/src" -Recurse -Filter "*.rs"
foreach ($f in $files) { $f.LastWriteTime = Get-Date }
$files = Get-ChildItem -Path "crates/winxime-config/src" -Recurse -Filter "*.rs"
foreach ($f in $files) { $f.LastWriteTime = Get-Date }
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# 2. Copy data files (librime/data/minimal -> target/release/data)
Write-Host "Step 2: Copying data files..." -ForegroundColor Yellow
if (Test-Path "target\release\data") {
    Remove-Item "target\release\data" -Recurse -Force
}
Copy-Item "librime\data\minimal" "target\release\data" -Recurse

# 2.5 Copy resources files (resources -> target/release/resources)
Write-Host "Step 2.5: Copying resources files..." -ForegroundColor Yellow
if (Test-Path "target\release\resources") {
    Remove-Item "target\release\resources" -Recurse -Force
}
Copy-Item "resources" "target\release\resources" -Recurse

# 3. Harvest data files with heat
Write-Host "Step 3: Harvesting data files..." -ForegroundColor Yellow
heat dir "target\release\data" -o "crates\winxime-server\wix\data.wxs" -dr DataFolder -cg DataFiles -var var.DataDir -sreg -srd -ag

# 3.5 Harvest resources files with heat
Write-Host "Step 3.5: Harvesting resources files..." -ForegroundColor Yellow
heat dir "target\release\resources" -o "crates\winxime-server\wix\resources.wxs" -dr ResourcesFolder -cg ResourcesFiles -var var.ResourcesDir -sreg -srd -ag

# 4. Compile and link MSI
Write-Host "Step 4: Building MSI..." -ForegroundColor Yellow
if (-not (Test-Path "target\wix")) {
    New-Item "target\wix" -ItemType Directory -Force | Out-Null
}
candle -arch x64 "crates\winxime-server\wix\main.wxs" "crates\winxime-server\wix\data.wxs" "crates\winxime-server\wix\resources.wxs" -ext WixUIExtension -ext WixUtilExtension "-dCargoTargetBinDir=target\release" "-dDataDir=target\release\data" "-dResourcesDir=target\release\resources" "-dVersion=$Version" -out "target\wix\"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Candle failed!" -ForegroundColor Red
    exit 1
}
light "target\wix\main.wixobj" "target\wix\data.wixobj" "target\wix\resources.wixobj" -ext WixUIExtension -ext WixUtilExtension -cultures:zh-CN -loc "crates\winxime-server\wix\zh-cn.wxl" -out "target\wix\xime-$Version.msi"
$lightExit = $LASTEXITCODE

# 5. Check result
$msiPath = "target\wix\xime-$Version.msi"
if (Test-Path $msiPath) {
    $msi = Get-Item $msiPath
    Write-Host ""
    Write-Host "Success: $($msi.FullName)" -ForegroundColor Green
    Write-Host "Size: $([math]::Round($msi.Length / 1MB, 2)) MB" -ForegroundColor White
    Write-Host ""
    Write-Host "Install: msiexec /i $($msi.FullName)" -ForegroundColor Yellow
} else {
    Write-Host "MSI build failed! (light exit code: $lightExit)" -ForegroundColor Red
    exit 1
}