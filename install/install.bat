@echo off
REM Double-click this file to install calc_lang.
REM It runs install.ps1 (in the same folder) without changing your PowerShell settings.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
pause
