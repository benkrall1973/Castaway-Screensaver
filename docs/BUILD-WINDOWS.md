# Castaway_Screensaver V1.0.4 Windows Build Guide

This guide builds the Windows desktop application from the complete GPL source in
this repository. It does not provide, download, or redistribute the original
Johnny Castaway resource files.

## Prerequisites

Install the Rust MSVC toolchain and the Visual Studio C++ build tools. The MSVC
toolchain is required for a reliable local Windows build; this checkout's default
GNU toolchain may fail at link time when `dlltool.exe` is not available.

```powershell
rustup toolchain install 1.99.1-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc --toolchain 1.99.1-x86_64-pc-windows-msvc
```

Start a Developer PowerShell for Visual Studio, then run the commands below from
the repository root. The standard MSVC linker is supplied by the Visual Studio C++ build tools.

```powershell
rustup run 1.99.1-x86_64-pc-windows-msvc cargo build --locked --release -p castaway_screensaver --target x86_64-pc-windows-msvc
```

The resulting application is:

```text
target\x86_64-pc-windows-msvc\release\castaway_screensaver.exe
```

## Run with your own data

Place your legally obtained original data beside the executable, in a `data`
subdirectory, or point to it explicitly:

```powershell
.\target\x86_64-pc-windows-msvc\release\castaway_screensaver.exe --windowed --data C:\path\to\JohnnyCastawayData
```

The application looks for `RESOURCE.MAP` and `RESOURCE.001`. It can also use the
supported original archive formats described in [INSTALL.md](../INSTALL.md).

## Personal-use embedded-data build

An embedded build contains copyrighted Sierra/Dynamix data. Build it only from
your own lawful copy and do not publish the resulting executable.

```powershell
$env:CASTAWAY_SCREENSAVER_EMBED_DATA = 'C:\path\to\your\original\data'
rustup run 1.99.1-x86_64-pc-windows-msvc cargo build --locked --release -p castaway_screensaver --features embed-data --target x86_64-pc-windows-msvc -j 1
```

## Checks

Run source checks before publishing a change:

```powershell
cargo fmt --check
cargo check -p castaway_screensaver
```

The Windows icon is optional. A warning about a missing resource compiler
(`rc.exe` or `windres`) does not prevent the executable from running; it only
means Windows will show a generic icon.
