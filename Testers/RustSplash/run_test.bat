@echo off
echo Building and starting Splash Screen...
cargo build
start "" "target\debug\fast_splash.exe"

echo Waiting for socket to initialize...
timeout /t 1 /nobreak > nul

echo Starting Tester...
cargo run --bin tester

echo Test complete!
pause
