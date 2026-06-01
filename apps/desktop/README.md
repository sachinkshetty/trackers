# Desktop App

Tauri desktop shell for the tracker cleaner.

## Local Run

```powershell
cd apps\desktop
npm install
npm run tauri -- info
npm run tauri -- dev
```

## Structure

- `src-tauri` contains the Rust backend and Tauri configuration.
- `src` contains the Vite frontend that invokes narrow scanner commands.
- `crates/scanner-core` provides the reusable discovery and cleanup logic.
