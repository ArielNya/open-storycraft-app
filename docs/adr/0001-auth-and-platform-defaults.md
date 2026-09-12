# ADR 0001 — Auth stance and platform defaults

**Status:** accepted for phase 1  
**Date:** 2026-09-12

Phase 1 is a desktop CLI with zero network. These decisions are locked so later crates (`storycraft-auth`, `storycraft-llm`, `storycraft-app`) do not get redesigned mid-flight.

## Auth

Two providers. They are not interchangeable billing tracks.

1. **OpenAI-compatible API key is the reliable path.** Ship it first. Config is `base_url` + optional `api_key` + `api_style` (`chat_completions` | `responses`). This covers xAI console keys, OpenAI, Groq, local llama.cpp / LM Studio / Ollama shims.
2. **xAI device-code OAuth is second, behind a settings toggle.** Label it **“Sign in with xAI (community OAuth)”**. Do not call it SuperGrok login. SuperGrok / X Premium+ is a consumer subscription; console API keys are a separate bill. OAuth can return 403 tier-denied even after a successful login.
3. **Do not scrape grok.com cookies.**
4. Client id, auth host, and scopes live in config, not compiled constants-only. Default client id is the public Grok-CLI ecosystem id (`b1a00492-073a-47ea-816f-4c329264a828`) until xAI publishes an official third-party client.
5. Tokens go in the OS keystore / Android Keystore. Never markdown, never logs.

Phase 2 implements the API-key client. OAuth follows with recorded HTTP fixtures. No live Grok calls in CI.

## Platform

| Question | Decision |
|---|---|
| Shell | Tauri 2 + Rust core + tiny Svelte UI. Not Compose, not Dioxus, not Electron. |
| Display name / applicationId | Open Storycraft / `dev.openstorycraft.app` |
| v1 targets | Android APK (sideload) **and** Linux/Windows desktop from the same `storycraft-app` crate. Core + CLI land first. |
| Optional overlays (`ao3-*`, `anti-slop-editor`) | Out of scope until phase 6. Stay off unless the user enables them. |
| Default model preference | `grok-4.6` when `/v1/models` lists it; otherwise the provider’s first flagship-like id. Per-skill routing is later. |
| Book location | CLI: a path argument. Phone: user-picked folder (SAF) from day one, app-private storage as fallback. Writers need the files. |

## Skills on disk

The unpacked library lives at repo-root `open-storycraft/` (42 craft skills + orchestrator). That **is** the vendored pack. Do not duplicate it under `skills/`. The CLI looks for `open-storycraft/` or `skills/` walking up from cwd, or `STORYCRAFT_SKILLS`.

## Non-negotiable host contract

One skill, one job, confirmation gate. Status board is read from disk every time. The app does not run a skill “in spirit.”
