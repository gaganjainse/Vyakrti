@echo off
title Vyākṛti IDE Launcher
cd /d "%~dp0"
powershell -NoExit -ExecutionPolicy Bypass -File "start-ide.ps1"
