@echo off
setlocal EnableExtensions

if "%~1"=="" goto :usage

if /I "%~1"=="setup" goto :setup
if /I "%~1"=="test" goto :test
if /I "%~1"=="audit" goto :audit
if /I "%~1"=="demo" goto :demo
if /I "%~1"=="clean" goto :clean
if /I "%~1"=="fixtures" goto :fixtures
if /I "%~1"=="bench" goto :bench

echo Unknown target: %~1
goto :usage

:setup
echo Setting up workspace...
call cargo fetch --manifest-path coh-node\Cargo.toml || exit /b 1
call cargo fetch --manifest-path ape\Cargo.toml || exit /b 1
pushd coh-dashboard || exit /b 1
call npm ci || (popd & exit /b 1)
popd
exit /b 0

:test
echo Running all tests...
pushd coh-node || exit /b 1
call cargo fmt --check || (popd & exit /b 1)
call cargo test --all || (popd & exit /b 1)
popd
exit /b 0

:audit
echo Running cargo audit...
pushd coh-node || exit /b 1
call cargo audit || (popd & exit /b 1)
popd
exit /b 0

:demo
echo Building dashboard and preparing demo...
pushd coh-dashboard || exit /b 1
call npm run build || (popd & exit /b 1)
popd
exit /b 0

:clean
echo Cleaning workspace...
pushd coh-node || exit /b 1
call cargo clean || (popd & exit /b 1)
popd
pushd ape || exit /b 1
call cargo clean || (popd & exit /b 1)
popd
powershell -Command "if (Test-Path 'coh-dashboard/node_modules') { Remove-Item -Recurse -Force 'coh-dashboard/node_modules' }; if (Test-Path 'coh-dashboard/dist') { Remove-Item -Recurse -Force 'coh-dashboard/dist' }" || exit /b 1
exit /b 0

:fixtures
echo Rebuilding fixtures...
echo Fixture rebuild target is reserved for Phase 2 implementation.
exit /b 0

:bench
echo Running benchmarks...
pushd coh-node || exit /b 1
call cargo bench || (popd & exit /b 1)
popd
exit /b 0

:usage
echo Usage: make ^<setup^|test^|audit^|demo^|clean^|fixtures^|bench^>
exit /b 1
