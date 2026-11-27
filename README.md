# rea-launcher (Rust port)

A Rust rewrite of [protonspy/opencode-rea](https://github.com/protonspy/opencode-rea), with the same
purpose: one command that runs OpenCode with an abliterated model (NanoGPT or NEAR AI) and the REA
reverse-engineering MCP server, on a hash-pinned toolchain.

## Build

```powershell
cargo build --release
```

Binary: `target/release/rea-launcher.exe`.

## Usage

```powershell
rea-launcher -- run "explain how C:\samples\app.exe checks its licence"
rea-launcher --provider nearai
rea-launcher --ask
rea-launcher --print-config
rea-launcher setup --ghidra 'C:\Tools\ghidra_12.2_PUBLIC' --java-home 'C:\Tools\jdk-25.0.5.1+2'
rea-launcher logout
rea-launcher version
```

## Notes

- Uses the already-installed toolchain under `%LOCALAPPDATA%\rea-launcher` when present.
- Reads legacy credentials created by the Go version (Windows Credential Manager).
- Ignores Ctrl+C in the launcher and leaves it to OpenCode.
