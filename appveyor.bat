if "%PLATFORM%" == "x86" set RUST_INSTALL=i686-pc-windows-msvc
if "%PLATFORM%" == "x64" set RUST_INSTALL=x86_64-pc-windows-msvc
appveyor AddMessage "Platform rust: %RUST_INSTALL%"
appveyor DownloadFile "https://static.rust-lang.org/dist/rust-nightly-%RUST_INSTALL%.exe" -FileName rust-install.exe
"./rust-install.exe" /VERYSILENT /NORESTART /DIR="C:\Rust\"
SET PATH=%PATH%;C:\Rust\bin
rustc -V
cargo -V

appveyor DownloadFile http://www.npcglib.org/~stathis/downloads/openssl-1.0.1s-vs2015.7z -FileName openssl.7z
mkdir C:\openssl
7z x openssl.7z -oC:/openssl/ -y
set DEP_OPENSSL_INCLUDE=C:\openssl\openssl-1.0.1s-vs2015\include\
if "%PLATFORM%" == "x64" set OPENSSL_EXT=64
cp C:\openssl\openssl-1.0.1s-vs2015\lib%OPENSSL_EXT%\libeay32MD.lib C:\Rust\lib\rustlib\%RUST_INSTALL%\lib\eay32.lib
cp C:/openssl/openssl-1.0.1s-vs2015/lib%OPENSSL_EXT%\ssleay32MD.lib C:\Rust\lib\rustlib\%RUST_INSTALL%\lib\ssl32.lib

appveyor DownloadFile https://www.libsdl.org/release/SDL2-devel-2.0.4-VC.zip -FileName sdl2-dev.zip
mkdir C:\sdl2
7z x sdl2-dev.zip -oC:\sdl2\ -y
cp C:\sdl2\SDL2-2.0.4\lib\%PLATFORM%\SDL2.lib C:\Rust\lib\rustlib\%RUST_INSTALL%\lib\SDL2.lib

cargo build
mkdir dist-debug
cp target\debug\steven.exe dist-debug
cp C:\sdl2\SDL2-2.0.4\lib\%PLATFORM%\SDL2.dll dist-debug
cp C:\openssl\openssl-1.0.1s-vs2015\bin%OPENSSL_EXT%\libeay32MD.dll dist-debug\libeay32MD.dll
cp C:\openssl\openssl-1.0.1s-vs2015\bin%OPENSSL_EXT%\ssleay32MD.dll dist-debug\ssleay32MD.dll

cargo build --release
mkdir dist
cp target\release\steven.exe dist
cp C:\sdl2\SDL2-2.0.4\lib\%PLATFORM%\SDL2.dll dist
cp C:\openssl\openssl-1.0.1s-vs2015\bin%OPENSSL_EXT%\libeay32MD.dll dist\libeay32MD.dll
cp C:\openssl\openssl-1.0.1s-vs2015\bin%OPENSSL_EXT%\ssleay32MD.dll dist\ssleay32MD.dll
