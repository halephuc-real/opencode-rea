# rea-launcher

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

## Build

```powershell
cargo build --release
```

Release binary:

```powershell
.\target\release\rea-launcher.exe
```

Or build for all targets via GitHub Actions (see `.github/workflows` if added).

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

- Run in a disposable VM with no secrets for untrusted targets.
- Use a NanoGPT key with a low spend cap.
- Start sessions in an empty working directory.
```

Then commit & push.<|end|><|start|> assistant to=shell<|channel|>analysis <|constrain|>xml<|message|><rfl:parameter name="command">
cd C:\Users\Lusquen\Desktop\opencode-rea; git add README.md; git commit -m "Rewrite README as standalone project" 2>&1 | Select-Object -Last 5; git push 2>&1 | Select-Object -Last 5