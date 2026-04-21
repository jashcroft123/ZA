@echo off
echo Building and starting Splash Screen...
cargo build --release
start "" "target\debug\fast_splash.exe"

echo Waiting for socket to initialize...
timeout /t 1 /nobreak > nul

echo Starting Tester...
cargo run --bin tester --release

echo Test complete!
pause
