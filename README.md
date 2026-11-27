# opencode-rea

A standalone Rust CLI that launches OpenCode with an abliterated model and the REA
reverse-engineering MCP server attached, on a hash-pinned toolchain.

## Features

- One command to start a reverse-engineering session.
- Managed toolchain: Node.js 24.22.3, REA 6.10.1, OpenCode 1.19.5, verified by hash.
- Providers: NanoGPT (default) or NEAR AI.
- Automatic key handling (env, credential store, OAuth/PKCE, hidden prompt).
- REA MCP wiring, skills, and reverse-engineering `AGENTS.md`.
- Reviewed permission rules with `--ask` or auto-approve.
- Ghidra/JDK engine compatibility checks.
- Length-preserving PHX LAUNCHER branding for the managed OpenCode TUI.

## Build

```powershell
cargo build --release
```

Release binary:

```powershell
.\target\release\opencode-rea.exe
```

## Usage

```powershell
opencode-rea -- run "explain how C:\samples\app.exe checks its licence"
opencode-rea --provider nearai
opencode-rea --ask
opencode-rea --print-config
opencode-rea setup --ghidra 'C:\Tools\ghidra_12.2_PUBLIC' --java-home 'C:\Tools\jdk-25.0.5.1+2'
opencode-rea brand --status
opencode-rea brand
opencode-rea brand --restore
opencode-rea logout
opencode-rea version
```

Use `--no-brand` to launch without applying the PHX LAUNCHER patch.

## Branding

Before launching, `opencode-rea` patches the managed OpenCode binary in place:

- the large TUI logo renders **PHX LAUNCHER**;
- the terminal title is set to `PHX LAUNCHER`;
- session titles use the `PHX | ...` prefix.

The patch only changes display strings and the logo module. It keeps the Bun
standalone bundle length identical, so the executable still starts normally.
The original binary is preserved as `opencode.exe.orig` next to the managed
OpenCode executable. `opencode-rea brand --restore` copies it back.

If another OpenCode instance is holding the executable open, close it and run
`opencode-rea brand` again.

## Notes

- Run in a disposable VM with no secrets for untrusted targets.
- Use a NanoGPT key with a low spend cap.
- Start sessions in an empty working directory.
