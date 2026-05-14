# Complete uninstall of MSI-installed Xime

Write-Host "Uninstalling Xime..." -ForegroundColor Yellow

# 1. Stop server
Write-Host "Step 1: Stopping server..." -ForegroundColor Yellow
Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "winxime-setup" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# 2. Uninstall all MSI products
Write-Host "Step 2: Uninstalling MSI..." -ForegroundColor Yellow
$uninstallKeys = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue
$ximeProducts = $uninstallKeys | Where-Object { $_.DisplayName -eq "Xime" }

if ($ximeProducts) {
    Write-Host "  Found: $($ximeProducts.DisplayName -join ' ') ($($ximeProducts.DisplayVersion -join ' '))" -ForegroundColor Gray
    foreach ($product in $ximeProducts) {
        $uninstallString = $product.UninstallString
        if ($uninstallString -match "msiexec") {
            $productCode = ($uninstallString -split " ")[1]
            Write-Host "  Uninstalling $productCode..." -ForegroundColor Gray
            $proc = Start-Process msiexec -ArgumentList "/x $productCode /passive /norestart" -PassThru
            if (-not $proc.WaitForExit(30000)) {
                Write-Host "  Timeout, killing process..." -ForegroundColor Yellow
                $proc.Kill()
            }
            Start-Sleep -Seconds 2
        }
    }
} else {
    Write-Host "  Xime not found in registry" -ForegroundColor Gray
}

# 3. Clean residual files
Write-Host "Step 3: Cleaning residual files..." -ForegroundColor Yellow
Remove-Item "C:\Program Files\Xime" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Windows\System32\winxime_tsf.dll" -Force -ErrorAction SilentlyContinue

# 4. Clean registry (TSF registration)
Write-Host "Step 4: Cleaning registry..." -ForegroundColor Yellow
$clsid = "{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}"
reg delete "HKLM\SOFTWARE\Classes\CLSID\$clsid" /f 2>$null
reg delete "HKLM\SOFTWARE\Microsoft\CTF\TIP\$clsid" /f 2>$null
reg delete "HKLM\SOFTWARE\Classes\AppID\$clsid" /f 2>$null
reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v XimeServer /f 2>$null
reg delete "HKCU\SOFTWARE\Microsoft\CTF\TIP\$clsid" /f 2>$null
reg delete "HKCU\Software\Xime" /f 2>$null

# Remove stale MSI registry entries
$staleProducts = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -eq "Xime" }
foreach ($stale in $staleProducts) {
    $guid = $stale.PSChildName
    Write-Host "  Removing stale registry entry: $guid" -ForegroundColor Gray
    Remove-Item "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$guid" -Force -ErrorAction SilentlyContinue
}

# 5. Verify cleanup
Write-Host ""
Write-Host "Verification:" -ForegroundColor Cyan
$remaining = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -eq "Xime" }
if ($remaining) {
    Write-Host "  Warning: MSI still registered" -ForegroundColor Yellow
} else {
    Write-Host "  MSI products: None (OK)" -ForegroundColor Green
}

if (Test-Path "C:\Program Files\Xime") {
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
Write-Host "Done! You may need to restart Windows to complete TSF cleanup." -ForegroundColor Green