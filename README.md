# CorteX

CorteX is a Windows-first desktop writing assistant built with Tauri v2, Rust,
React, TypeScript, and SQLite.

## What Is Included

- Main desktop window matching the supplied CorteX interface.
- Floating rewrite popup without the traffic-light circles and with `Output Text`
  as the primary text section.
- System tray menu for opening CorteX, rewriting clipboard text, favorites,
  settings, pausing shortcuts, and exiting.
- Global shortcuts:
  - `Ctrl + Alt + Z`: capture selected text, rewrite it, and show the popup.
  - `Ctrl + 1`: fix grammar directly in the active app.
  - `Ctrl + 2`: rewrite professionally directly in the active app.
- SQLite settings and rewrite history.
- Offline rewrite utilities plus provider adapters for OpenAI, OpenRouter,
  Gemini, Anthropic Claude, and Ollama.

## Run On Windows

```powershell
npm.cmd install --cache .npm-cache
npm.cmd run tauri:dev
```

## Build

```powershell
npm.cmd run tauri:build
```

If PowerShell blocks `npm`, use `npm.cmd` as shown above.
