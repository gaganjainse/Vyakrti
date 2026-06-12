# Vyakrti IDE - One-click Launcher
# Launches both the Rust backend and the Vite frontend dev server.

$BackendDir = Join-Path $PSScriptRoot "backend"
$FrontendDir = Join-Path $PSScriptRoot "frontend"
$WorkspaceRoot = Split-Path $PSScriptRoot -Parent
$url = "http://localhost:5173"

Write-Host "+---------------------------------------------+" -ForegroundColor Cyan
Write-Host "|     Vyakrti Workbench Launcher              |" -ForegroundColor Cyan
Write-Host "+---------------------------------------------+" -ForegroundColor Cyan
Write-Host "|  Backend  : cargo run (port 8080)          |" -ForegroundColor Cyan
Write-Host "|  Frontend : npm run dev  (port 5173)       |" -ForegroundColor Cyan
Write-Host "+---------------------------------------------+" -ForegroundColor Cyan
Write-Host ""

# --- Verify prerequisites ---
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Host "[X] Rust / cargo not found. Install from https://rustup.rs" -ForegroundColor Red
  exit 1
}
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
  Write-Host "[X] Node.js / npm not found. Install from https://nodejs.org" -ForegroundColor Red
  exit 1
}

# --- Install frontend dependencies if missing ---
if (-not (Test-Path (Join-Path $FrontendDir "node_modules"))) {
  Write-Host "[.] Installing frontend dependencies..." -ForegroundColor Yellow
  Push-Location $FrontendDir
  npm install
  Pop-Location
}

# --- Kill leftover processes on ports 8080 and 5173 ---
$ports = @(8080, 5173)
foreach ($port in $ports) {
  $conn = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
  if ($conn -and $conn.OwningProcess) {
    $proc = Get-Process -Id $conn.OwningProcess -ErrorAction SilentlyContinue
    if ($proc) {
      $proc | Stop-Process -Force -ErrorAction SilentlyContinue
      Write-Host "[.] Killed process on port $port" -ForegroundColor Gray
    }
  }
}

Start-Sleep -Seconds 2

# --- Launch backend with log redirection ---
Write-Host "[.] Starting Vyakrti backend..." -ForegroundColor Yellow
$backendOut = Join-Path $PSScriptRoot "backend_out.log"
$backendErr = Join-Path $PSScriptRoot "backend_err.log"
$backendPid = (Start-Process -WindowStyle Hidden -FilePath "cmd.exe" -ArgumentList "/c set VYAKRTI_WORKSPACE=$WorkspaceRoot&& cargo run > ""$backendOut"" 2> ""$backendErr""" -WorkingDirectory $BackendDir -PassThru).Id
Write-Host "[V] Backend PID: $backendPid  (logs: backend_out.log / backend_err.log)" -ForegroundColor Green

# --- Launch frontend with log redirection ---
Write-Host "[.] Starting Vite dev server..." -ForegroundColor Yellow
$frontendOut = Join-Path $PSScriptRoot "frontend_out.log"
$frontendErr = Join-Path $PSScriptRoot "frontend_err.log"
$frontendPid = (Start-Process -WindowStyle Hidden -FilePath "cmd.exe" -ArgumentList "/c set VITE_BACKEND_URL=http://127.0.0.1:8080&& npm run dev > ""$frontendOut"" 2> ""$frontendErr""" -WorkingDirectory $FrontendDir -PassThru).Id
Write-Host "[V] Frontend PID: $frontendPid  (logs: frontend_out.log / frontend_err.log)" -ForegroundColor Green

# --- Poll until frontend is reachable ---
Write-Host ""
Write-Host "[.] Waiting for frontend to be ready..." -ForegroundColor Yellow
$maxWait = 30
for ($i = 0; $i -lt $maxWait; $i++) {
  Start-Sleep -Seconds 1
  try {
    $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
      Write-Host "[V] Frontend is ready!" -ForegroundColor Green
      break
    }
  } catch {
    if ($i -eq 0) { Write-Host "[.] Waiting for server..." -NoNewline -ForegroundColor Gray }
    Write-Host "." -NoNewline -ForegroundColor Gray
  }
  if ($i -eq $maxWait - 1) { Write-Host "" }
}

Write-Host ""
Write-Host "-----------------------------------------------" -ForegroundColor Cyan
Write-Host "  IDE ready at:  $url" -ForegroundColor Cyan
Write-Host "-----------------------------------------------" -ForegroundColor Cyan
Write-Host ""

# --- Open browser ---
Write-Host "[.] Opening browser..." -ForegroundColor Yellow
Start-Process $url

Write-Host ""
Write-Host "Backend PID: $backendPid  |  Frontend PID: $frontendPid" -ForegroundColor Gray
Write-Host "To stop both servers, run:  taskkill /F /PID $backendPid /PID $frontendPid" -ForegroundColor Gray
