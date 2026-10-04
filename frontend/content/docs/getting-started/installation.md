Princi v0.1 builds native Windows x86-64 executables. A local setup needs the Rust toolchain for the compiler itself, LLVM/Clang for verified Windows object generation, and MinGW-w64 GCC for the final link.

> **Windows-only target:** These steps set up the supported v0.1 target. The compiler does not currently produce Linux, macOS, or WebAssembly programs.

## Download the v0.1 compiler

<!-- princi-release-downloads -->

## Prerequisites

| Tool | Required setup | Verify |
| --- | --- | --- |
| Rust and Cargo | Stable Rust toolchain installed through [rustup](https://rust-lang.org/tools/install/). PowerShell must be able to resolve both commands. On Windows, rustup may prompt for the Visual Studio C++ Build Tools required by the Rust host toolchain. | `rustc --version` and `cargo --version` |
| LLVM/Clang | Clang 15 or newer, installed with LLVM IR input and the X86 target; use the [official LLVM releases](https://releases.llvm.org/). Add the LLVM `bin` directory to Windows `PATH`. | `clang --version` and `clang --print-targets` |
| MinGW-w64 GCC | x86-64 Windows GNU compiler/linker. The documented setup uses MSYS2 UCRT64. Add its `ucrt64\bin` directory (normally `C:\msys64\ucrt64\bin`) to Windows `PATH`. | `gcc -dumpmachine` |

The Clang target list must include **X86**. `gcc -dumpmachine` must print `x86_64-w64-mingw32`; another GCC installation earlier on `PATH` may report a different target and cannot link Princi's output.

## Install MinGW-w64 through MSYS2 UCRT64

Install [MSYS2](https://www.msys2.org/) if it is not already installed. Open the **MSYS2 UCRT64** terminal and update packages:

```sh
pacman -Syu
```

If the update asks you to close the terminal, close it, reopen UCRT64, and complete the update as prompted. Then install the x86-64 GCC package:

```sh
pacman -S mingw-w64-ucrt-x86_64-gcc
```

Add `C:\msys64\ucrt64\bin` to the Windows `PATH` (adjust the drive or MSYS2 install directory if yours differs). Open a fresh PowerShell window and verify the selected tools:

```powershell
Get-Command clang
Get-Command gcc
clang --version
clang --print-targets
gcc -dumpmachine
```

Use `where.exe clang` or `where.exe gcc` to see every matching executable on `PATH` if the wrong installation is selected.

## Clone and build the compiler

In PowerShell:

```powershell
git clone https://github.com/0xPolybit/principl.git
cd principl
cargo build
cargo build --release
```

The debug compiler is `target\debug\princi.exe`; the release compiler is `target\release\princi.exe`. You can call either binary directly from the repository root, for example:

```powershell
.\target\debug\princi.exe build .\examples\modules.prnc
```

## Install the `princi` command

Cargo can install the compiler executable from the cloned repository:

```powershell
cargo install --path .
```

This places `princi.exe` in Cargo's user binary directory, normally `%USERPROFILE%\.cargo\bin`. If PowerShell reports that `princi` is not recognized, add that directory to your Windows `PATH`, open a new terminal, and check:

```powershell
Get-Command princi
```

`cargo install --path .` installs the CLI; it does not install LLVM/Clang or MinGW-w64. Those native build prerequisites must remain available separately.

## Tool selection

By default the compiler starts `clang` and `gcc` by name, using the executables found on `PATH`. If a valid tool is installed elsewhere, set the corresponding environment variable to its executable path before building:

```powershell
$env:PRINCI_CLANG = 'C:\Program Files\LLVM\bin\clang.exe'
$env:PRINCI_CC = 'C:\msys64\ucrt64\bin\gcc.exe'
```

`PRINCI_CLANG` selects the LLVM/Clang executable; `PRINCI_CC` selects MinGW-w64 GCC. The configured GCC must still report the required `x86_64-w64-mingw32` target.

## Common setup problems

| Diagnostic or symptom | What to check |
| --- | --- |
| `princi` is not recognized | Confirm `%USERPROFILE%\.cargo\bin` is on `PATH`, or run `target\debug\princi.exe` from the clone. Start a new terminal after changing `PATH`. |
| `E0401` LLVM/Clang could not be started | Install Clang 15+ with LLVM IR and X86 support, check `Get-Command clang`, and make sure its `bin` directory is on `PATH`; alternatively set `PRINCI_CLANG`. |
| `E0403` Windows linker is unavailable or has the wrong target | Check `Get-Command gcc` and `gcc -dumpmachine`. Install the MSYS2 UCRT64 x86-64 package and ensure its `ucrt64\bin` directory is selected, or set `PRINCI_CC`. |
| `E0404` Windows linking failed | Verify the object-generation toolchain and GCC installation, and ensure the output directory is writable. The compiler reports a linker failure rather than treating it as a successful build. |
| `rustc` or `cargo` is not recognized, or Cargo cannot build the compiler | Finish the stable toolchain setup with rustup and reopen PowerShell so the Rust commands are on `PATH`. On Windows, install the Visual Studio C++ Build Tools if requested by the Rust host toolchain. |

The compiler runs Clang with the explicit `x86_64-w64-windows-gnu` target, then checks that GCC targets `x86_64-w64-mingw32`. See the [Windows toolchain reference](/docs/compiler/windows-toolchain) for the native backend details and [diagnostics](/docs/compiler/diagnostics) for compiler error categories.
