param(
    [string]$Command = "build"
)

function Build {
    Write-Host "Building all components..." -ForegroundColor Cyan
    cargo build
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    Write-Host "Build completed successfully" -ForegroundColor Green
}

function RunServer {
    Write-Host "Starting server..." -ForegroundColor Cyan
    cargo run -p winxime-server
}

function RunDev {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Winxime Development Workflow" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""

    $proc = Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue
    if ($proc) {
        Write-Host "Stopping existing server (PID: $($proc.Id))..." -ForegroundColor Yellow
        Stop-Process -Id $proc.Id -Force
        Start-Sleep -Milliseconds 500
    }

    Write-Host "[1/3] Building..." -ForegroundColor Cyan
    cargo build
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }

    Write-Host ""
    Write-Host "[2/3] Starting server..." -ForegroundColor Cyan
    $serverExe = "target\debug\winxime-server.exe"
    if (-not (Test-Path $serverExe)) { throw "Server not found after build" }

    Start-Process -FilePath $serverExe -WindowStyle Normal
    Start-Sleep -Milliseconds 1000

    $proc = Get-Process -Name "winxime-server" -ErrorAction SilentlyContinue
    if (-not $proc) { throw "Server failed to start" }
    Write-Host "Server started (PID: $($proc.Id))." -ForegroundColor Green
    Write-Host ""

    Write-Host "[3/3] To test the input method:" -ForegroundColor Cyan
    Write-Host "  Register TSF DLL (elevated PowerShell):" -ForegroundColor White
    Write-Host "    regsvr32 target\debug\winxime_tsf.dll" -ForegroundColor Gray
    Write-Host "  Then switch to Xime input method in any app" -ForegroundColor White
    Write-Host ""
    Write-Host "To stop the server:" -ForegroundColor Yellow
    Write-Host "  taskkill /F /IM winxime-server.exe" -ForegroundColor Gray
    Write-Host "========================================" -ForegroundColor Cyan
}

function InstallDll {
    $dllPath = Resolve-Path "target\debug\winxime_tsf.dll" -ErrorAction Stop
    Write-Host "Registering $dllPath ..." -ForegroundColor Cyan
    Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "`"$dllPath`""

    Write-Host "Enabling Xime input method..." -ForegroundColor Cyan
    reg add "HKCU\Software\Microsoft\CTF\TIP\{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}\LanguageProfile\0x00000804\{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}" /ve /f 2>&1

    Write-Host "TSF DLL registered successfully" -ForegroundColor Green
    Write-Host "Press Win+Space to switch to Xime" -ForegroundColor Yellow
}

function UninstallDll {
    $dllPath = Resolve-Path "target\debug\winxime_tsf.dll" -ErrorAction Stop
    Write-Host "Unregistering $dllPath ..." -ForegroundColor Cyan
    Start-Process -Verb RunAs -Wait -FilePath "regsvr32.exe" -ArgumentList "/u `"$dllPath`""

    Write-Host "Disabling Xime input method..." -ForegroundColor Cyan
    reg delete "HKCU\Software\Microsoft\CTF\TIP\{5C1E4D8A-F3B2-4A7E-9CD1-2A3B4C5D6E7F}" /f 2>&1

    Write-Host "TSF DLL unregistered successfully" -ForegroundColor Green
}

try {
    switch ($Command) {
        "build"     { Build }
        "run"       { RunServer }
        "run-dev"   { RunDev }
        "install"   { InstallDll }
        "uninstall" { UninstallDll }
        default {
            Write-Host "Usage: .\scripts\dev.ps1 <command>" -ForegroundColor Yellow
            Write-Host "Commands:" -ForegroundColor Yellow
            Write-Host "  build       - Build all components" -ForegroundColor Gray
            Write-Host "  run         - Start the server (Ctrl+C to stop)" -ForegroundColor Gray
            Write-Host "  run-dev     - Build + start server" -ForegroundColor Gray
            Write-Host "  install     - Register TSF DLL (elevated)" -ForegroundColor Gray
            Write-Host "  uninstall   - Unregister TSF DLL (elevated)" -ForegroundColor Gray
            exit 1
        }
    }
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}
