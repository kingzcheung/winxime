# Rebuild and test winxime (development workflow)

$iconPath = "$PSScriptRoot\resource\icon.ico"
$registerExe = "$PSScriptRoot\target\debug\winxime-tsf-register.exe"
$userDataDir = "$PSScriptRoot\target\debug\user-data"
$sharedDataDir = "$PSScriptRoot\librime\data\minimal"

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

Write-Host "Step 4.5: Setting up user data directory..." -ForegroundColor Yellow
if (Test-Path $userDataDir) {
    Remove-Item $userDataDir -Recurse -Force
}
New-Item -ItemType Directory -Path $userDataDir -Force | Out-Null

Copy-Item $sharedDataDir $userDataDir -Recurse

$exclude = @('.git', '.github', 'imgs', 'README.md', '.gitignore', 'macOS-*', '*.command', 'LICENSE', 'squirrel.custom.yaml', 'trime.custom.yaml')
Get-ChildItem -Path "$PSScriptRoot\rime-wubi" -Recurse -File | Where-Object {
    $dir = $_.DirectoryName
    $name = $_.Name
    -not ($exclude | Where-Object { $dir -like "*$_*" -or $name -like $_ })
} | ForEach-Object {
    $relativePath = $_.FullName.Substring("$PSScriptRoot\rime-wubi".Length + 1)
    $destPath = "$userDataDir\$relativePath"
    $destDir = Split-Path -Parent $destPath
    if (-not (Test-Path $destDir)) {
        New-Item $destDir -ItemType Directory -Force | Out-Null
    }
    Copy-Item $_.FullName $destPath -Force
}
Write-Host "  User data directory ready: $userDataDir" -ForegroundColor Gray

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