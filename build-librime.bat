@echo off
REM Build librime for winxime
REM Run this script in Developer Command Prompt for VS 2022

setlocal

cd /d %~dp0

REM Check if already built
if exist "dist\rime.dll" (
    echo librime already built: dist\rime.dll
    goto :exit
)

REM Setup VS2022 environment
if exist "env.vs2022.bat" call env.vs2022.bat

REM Create env.bat if not exists
if not exist "env.bat" (
    echo Creating env.bat...
    copy env.bat.template env.bat
)

REM Install Boost if not present
if not defined BOOST_ROOT (
    echo Installing Boost...
    call install-boost.bat
    if errorlevel 1 goto :error
)

REM Build dependencies
echo Building dependencies...
call build.bat deps
if errorlevel 1 goto :error

REM Build librime
echo Building librime...
call build.bat librime
if errorlevel 1 goto :error

echo.
echo Build complete! Output: dist\rime.dll
echo.

goto :exit

:error
echo.
echo Build FAILED. Please check the error messages.
echo You may need to run this in "Developer Command Prompt for VS 2022"
echo.

:exit
endlocal