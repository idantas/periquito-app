# Periquito

A Tamagotchi-style parrot that lives in your MacBook notch and teaches you English while you code.

Periquito intercepts your prompts in Claude Code, analyzes your English grammar in real-time, and helps you learn through corrections, praise, and spaced repetition — all without breaking your flow.

## How it works

1. You write prompts in Claude Code (or any AI tool)
2. Periquito analyzes your English in the background
3. The parrot reacts — happy for good English, sad for mistakes
4. You get corrections with explanations and vocabulary tips
5. Over time, your mistakes become quiz questions (spaced repetition)
6. The parrot evolves as you improve (Egg → Chick → Parrot → Macaw → Phoenix)

## Stack

- **Frontend:** React 19 + TypeScript + Vite
- **Backend:** Rust (Tauri 2) with tokio async
- **Platform:** macOS notch integration via private API
- **Analysis:** Any AI provider (Claude, OpenAI, Ollama)
- **Data:** Local JSONL files — no cloud, no accounts

## Running

```bash
npm install
npm run dev
```

Requires: Rust toolchain, Xcode Command Line Tools, macOS 14+

## Credits

Inspired by [notchi](https://github.com/sk-ruban/notchi) by sk-ruban and [periquito](https://github.com/lucianfialho/periquito) by Lucian Fialho. Built as a ground-up rewrite with a different architecture and learning-focused direction.

## License

MIT
