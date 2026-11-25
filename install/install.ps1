# install.ps1 - installs calc_lang for the current user on Windows
# - copies calc_lang.exe to %LOCALAPPDATA%\Programs\calc_lang
# - adds a "Calc Lang" shortcut to the Start menu
# - adds the folder to your PATH so "calc_lang" works in any terminal

$ErrorActionPreference = "Stop"

# 1. Find calc_lang.exe: first next to this script, then in Downloads
$src = Get-ChildItem $PSScriptRoot -Recurse -Filter calc_lang.exe -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $src) {
    $src = Get-ChildItem "$env:USERPROFILE\Downloads" -Recurse -Filter calc_lang.exe -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
}
if (-not $src) {
    Write-Host "Could not find calc_lang.exe. Put this script in the same folder as calc_lang.exe and try again." -ForegroundColor Red
    exit 1
}
Write-Host "Found: $($src.FullName)"

# 2. Copy it to a permanent place
$dest = "$env:LOCALAPPDATA\Programs\calc_lang"
New-Item -ItemType Directory -Force $dest | Out-Null
Copy-Item $src.FullName $dest -Force
Unblock-File "$dest\calc_lang.exe"

# 3. Start menu shortcut
$shell = New-Object -ComObject WScript.Shell
$lnk = $shell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Calc Lang.lnk")
$lnk.TargetPath = "$dest\calc_lang.exe"
$lnk.WorkingDirectory = $dest
$lnk.Save()

# 4. Add to PATH (user only, no admin needed)
$path = [Environment]::GetEnvironmentVariable("Path", "User")
if ($path -notlike "*$dest*") {
    [Environment]::SetEnvironmentVariable("Path", "$path;$dest", "User")
}

Write-Host ""
Write-Host "Done! calc_lang is installed in $dest" -ForegroundColor Green
Write-Host "- Press the Windows key and type 'calc lang' to open it"
Write-Host "- Or open a NEW terminal and type: calc_lang"
