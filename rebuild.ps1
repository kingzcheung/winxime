# Rebuild and test winxime (development workflow)

$iconPath = "$PSScriptRoot\resource\icon.ico"
$registerExe = "$PSScriptRoot\target\debug\winxime-tsf-register.exe"
$exeDir = "$PSScriptRoot\target\debug"
$sharedDataDir = "$exeDir\data"
$userDataDir = "$exeDir\user-data"
$configSourceDir = "$exeDir\resources"
$rimeMinimalDir = "$PSScriptRoot\librime\data\minimal"
$rimeWubiDir = "$PSScriptRoot\rime-wubi"
$resourcesDir = "$PSScriptRoot\resources"

Write-Host "Step 0: Clearing old logs..." -ForegroundColor Yellow
Remove-Item "$env:TEMP\winxime\*.log" -Force -ErrorAction SilentlyContinue

Write-Host "Step 1: Stopping server..." -ForegroundColor Yellow
cargo run -p winxime-server -- /q 2>&1 | Out-Null
Start-Sleep -Seconds 3

$serverProcess = Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue
if ($serverProcess) {
    Write-Host "  Server still running, waiting for graceful shutdown..." -ForegroundColor Yellow
    Start-Sleep -Seconds 5
    $serverProcess = Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue
    if ($serverProcess) {
        Write-Host "  Force stopping server..." -ForegroundColor Red
        $serverProcess | Stop-Process -Force
        Start-Sleep -Seconds 2
    }
}

Write-Host "Step 2: Unregistering COM DLL..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/u", "/s", "$PSScriptRoot\target\debug\winxime_tsf.dll" -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

Write-Host "Step 3: Unregistering profile..." -ForegroundColor Yellow
if (Test-Path $registerExe) {
    Start-Process -Verb RunAs -Wait -FilePath $registerExe -ArgumentList "-u"
    Start-Sleep -Seconds 3
}

Write-Host "Step 4: Building..." -ForegroundColor Yellow
cargo build --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Step 4.5: Setting up data directories..." -ForegroundColor Yellow

# Shared data (RIME schemas) — fresh each build
if (Test-Path $sharedDataDir) {
    Remove-Item $sharedDataDir -Recurse -Force
}
New-Item -ItemType Directory -Path $sharedDataDir -Force | Out-Null
Copy-Item "$rimeMinimalDir\*" $sharedDataDir -Recurse
$exclude = @('.git', '.github', 'imgs', 'README.md', '.gitignore', 'macOS-*', '*.command', 'LICENSE', 'squirrel.custom.yaml', 'trime.custom.yaml')
Get-ChildItem -Path $rimeWubiDir -Recurse -File | Where-Object {
    $dir = $_.DirectoryName; $name = $_.Name
    -not ($exclude | Where-Object { $dir -like "*$_*" -or $name -like $_ })
} | ForEach-Object {
    $relativePath = $_.FullName.Substring("$rimeWubiDir".Length + 1)
    $destPath = "$sharedDataDir\$relativePath"
    $destDir = Split-Path -Parent $destPath
    if (-not (Test-Path $destDir)) { New-Item $destDir -ItemType Directory -Force | Out-Null }
    Copy-Item $_.FullName $destPath -Force
}
Write-Host "  Shared data: $sharedDataDir" -ForegroundColor Gray

# Config source dir (default xime.yaml for deploy)
New-Item -ItemType Directory -Path $configSourceDir -Force | Out-Null
Copy-Item "$resourcesDir\xime.yaml" "$configSourceDir\xime.yaml" -Force
Write-Host "  Config source: $configSourceDir" -ForegroundColor Gray

# System config (xime.yaml shipped with app)
Copy-Item "$resourcesDir\xime.yaml" "$sharedDataDir\xime.yaml" -Force
Write-Host "  System config: $sharedDataDir\xime.yaml" -ForegroundColor Gray

# User data (persistent — preserve across rebuilds)
if (-not (Test-Path $userDataDir)) {
    New-Item -ItemType Directory -Path $userDataDir -Force | Out-Null
}
Write-Host "  User data: $userDataDir" -ForegroundColor Gray

Write-Host "Step 5: Registering COM DLL (no profile)..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/s", "$PSScriptRoot\target\debug\winxime_tsf.dll"
Start-Sleep -Seconds 3

Write-Host "Step 6: Registering profile with icon..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath $registerExe -ArgumentList "-r", $iconPath
Start-Sleep -Seconds 3

Write-Host "Step 7: Enabling..." -ForegroundColor Yellow
Start-Process -Verb RunAs -Wait -FilePath $registerExe -ArgumentList "-i"
Start-Sleep -Seconds 2

Write-Host "Step 8: Starting server (debug mode)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cargo run -p winxime-server" -WindowStyle Normal
Start-Sleep -Seconds 5

Write-Host "`nDone!" -ForegroundColor Green
Write-Host "Server running in separate window (with console log)" -ForegroundColor Cyan
Write-Host "Test input in Notepad or any application" -ForegroundColor White