# Periquito-Tauri — Cross-platform Language Learning Companion

## What is this?
Rewrite of the native Swift Periquito app using **Tauri 2 + React 19 + TypeScript**. A Tamagotchi parrot that lives in the macOS notch, intercepts Claude Code prompts, analyzes English grammar via `claude -p`, and teaches through spaced repetition.

## Stack
- **Frontend:** React 19, TypeScript, Vite 6, Zustand 5
- **Backend:** Rust (Tauri 2), tokio async, serde
- **Platform:** macOS private API (cocoa/objc) for notch positioning
- **Analysis:** Claude Code CLI (`claude -p` subprocess)
- **Data:** JSONL files in `~/.english-learning/`

## Architecture
```
Claude Code Hook → periquito-hook.sh → /tmp/periquito.sock (Unix socket)
    → Rust: socket_server → state_machine → emotion_analyzer (claude -p)
    → Tauri emit → React UI (NotchLayout + SessionSprite + ChatTip)
```

## Key Directories
```
src/components/     → React UI (NotchLayout, GrassIsland, SessionSprite, ChatTip)
src/lib/            → IPC layer, sprite physics, tip parser
src/assets/sprites/ → 18 PNG sprite sheets (64x64 frames)
src-tauri/src/services/  → Rust services (socket, state machine, emotion, history)
src-tauri/src/models/    → Data models (HookEvent, PeriquitoState, ParrotLevel)
src-tauri/src/commands/  → IPC commands (hooks, settings)
src-tauri/src/platform/  → macOS window positioning
```

## Data Flow
1. Claude Code fires hook → `periquito-hook.sh` sends JSON to Unix socket
2. `socket_server.rs` receives HookEvent, dispatches to `state_machine.rs`
3. On UserPromptSubmit: `emotion_analyzer.rs` spawns `claude -p`, parses response
4. Result logged to `~/.english-learning/history.jsonl`
5. Emotion updated → `state-update` + `tips-update` emitted to React
6. React renders parrot sprite + correction/praise tips

## IPC Commands (React → Rust)
- `get_notch_geometry()`, `install_hooks()`, `uninstall_hooks()`, `is_hooks_installed()`
- `get_settings()`, `update_settings()`, `get_history_stats()`

## Events (Rust → React)
- `state-update` → { unified_state, session_count, is_any_analyzing }
- `tips-update` → { all_tips }

## Data Persistence
- `~/.english-learning/history.jsonl` — All corrections/praise (JSONL)
- `~/.english-learning/level.json` — XP, level, lastActiveDate
- `~/.english-learning/settings.json` — Sound, font, mute preferences
- `~/.english-learning/leitner-boxes.jsonl` — Spaced repetition state
- `/tmp/periquito.sock` — Runtime Unix socket

## Running
```bash
npm install
npm run dev     # Vite + Tauri dev mode (port 1420)
npm run build   # Production build
```

## Progress & Decisions
- **Progress tracking:** See memory docs in `~/.claude/projects/.../memory/PROGRESS.md`
- **Decision log:** See `~/.claude/projects/.../memory/DECISION.md`
- **Implementation plan:** See `~/.claude/plans/ancient-humming-puppy.md`

## Current Phase
Phase 1 — Stats & Progression UI (see progress doc for details)
