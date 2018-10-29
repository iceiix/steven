echo on
SetLocal EnableDelayedExpansion

REM This is the recommended way to choose the toolchain version, according to
REM Appveyor's documentation.
SET PATH=C:\Program Files (x86)\MSBuild\%TOOLCHAIN_VERSION%\Bin;%PATH%

SET PATH=C:\msys64\usr\bin;C:\mingw\bin;%PATH%
pacman --noconfirm -S git tar mingw-w64-x86_64-openssl mingw-w64-x86_64-SDL2 mingw-w64-x86_64-gcc

vcpkg install openssl:x86-windows
vcpkg install openssl:x64-windows

dir C:\OpenSSL
dir /r C:\OpenSSL
exit

set VCVARSALL="C:\Program Files (x86)\Microsoft Visual Studio %TOOLCHAIN_VERSION%\VC\vcvarsall.bat"

if [%Platform%] NEQ [x64] goto win32
set TARGET_ARCH=x86_64
set TARGET_PROGRAM_FILES=%ProgramFiles%
call %VCVARSALL% amd64
if %ERRORLEVEL% NEQ 0 exit 1
goto download

:win32
echo on
if [%Platform%] NEQ [Win32] exit 1
set TARGET_ARCH=i686
set TARGET_PROGRAM_FILES=%ProgramFiles(x86)%
call %VCVARSALL% amd64_x86
if %ERRORLEVEL% NEQ 0 exit 1
goto download

:download
REM vcvarsall turns echo off
echo on

mkdir build
set RUSTUP_URL=https://win.rustup.rs/%TARGET_ARCH%
set RUSTUP_EXE=build\rustup-init-%TARGET_ARCH%.exe
echo Downloading %RUSTUP_URL%...
powershell -Command "(New-Object Net.WebClient).DownloadFile('%RUSTUP_URL%', '%RUSTUP_EXE%')"
if %ERRORLEVEL% NEQ 0 (
  echo ...downloading rustup failed.
  exit 1
)

set TARGET=%TARGET_ARCH%-pc-windows-msvc
%RUSTUP_EXE% -y --default-host %TARGET% --default-toolchain %RUST%
if %ERRORLEVEL% NEQ 0 exit 1

set PATH=%USERPROFILE%\.cargo\bin;%cd%\windows_build_tools;%PATH%

if [%Configuration%] == [Release] set CARGO_MODE=--release

set

link /?
cl /?
rustc --version
cargo --version

cargo test -vv %CARGO_MODE%
if %ERRORLEVEL% NEQ 0 exit 1

REM Verify that `cargo build`, independent from `cargo test`, works; i.e.
REM verify that non-test builds aren't trying to use test-only features.
cargo build -vv %CARGO_MODE%
if %ERRORLEVEL% NEQ 0 exit 1
