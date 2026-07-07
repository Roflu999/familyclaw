@echo off
echo Checking for WebView2 Runtime...
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv >nul 2>&1
if %errorlevel% == 0 (
    echo WebView2 is installed. Starting OpenClaw Shell...
    start "" "OpenClaw-Shell.exe"
) else (
    echo.
    echo ================================================
    echo WebView2 Runtime is NOT installed.
    echo.
    echo OpenClaw Shell requires WebView2 to display its UI.
    echo Most Windows 10/11 PCs already have it.
    echo.
    echo To install it now, visit:
    echo https://developer.microsoft.com/en-us/microsoft-edge/webview2/
    echo.
    echo Or use the .msi installer instead, which bundles WebView2.
    echo ================================================
    echo.
    pause
)
