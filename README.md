# Open Storycraft

A local writing studio for the [Storycraft skill library](open-storycraft/README.md). Rust core, Tauri 2 shell, plain HTML/CSS/JS front end. It runs the craft skills one at a time along a planning spine — genre → audience → theme → synopsis → style → characters → voice → outline → scenes → psych → draft — and edits chapters with the editorial ladder.

Everything is read from disk, never from memory: the home screen is a status board derived from the files in the book folder. Model calls go to any OpenAI-compatible endpoint (xAI console keys, OpenAI, Groq, Together, OpenRouter, vLLM, llama.cpp, LM Studio, Ollama's OpenAI shim). Some skills run on device with no provider at all.

The long-form design notes are in [OPEN_STORYCRAFT_APP_PLAN.md](OPEN_STORYCRAFT_APP_PLAN.md).

## What works today

- **Status board** — twelve spine slots, each `yes` / `partial` / `no`, plus a `Next:` skill and the reason it is next. Hover a slot to see the file behind it; click it to run the skill that fills it.
- **Job cards** — every skill run streams into a preview. Nothing touches the Wiki until you save it; `Diff` and `Reject` are there for the change you did not want.
- **Book view** — file list and preview for the whole `Wiki/` and `Chapters/`, with the current file opened for you.
- **Write view** — chapters listed from the outline, each with a `Draft` button.
- **Edit view** — the ladder (cold read, dev edit, style review, AI-tells, prose, line, filter words, fragments, nominalizations, kill passes) plus the local burstiness report.
- **Storybible** — write the whole book out as one portable `storybible.md`, or unpack one into a book folder. See [Storybible](#storybible).
- **Command palette** — `Ctrl+K` lists every skill in the pack with its one-line description, filters as you type, marks the ones that run on device, and runs the one you pick. It is how you reach the skills the board does not recommend.
- **Connection profiles** — save several providers (NVIDIA NIM, Google AI Studio, a local LM Studio…), each with its own URL, models and key, and switch between them in Settings. See [Connection profiles](#connection-profiles).
- **Model picker** — the settings screen asks your provider what it serves and offers it in a dropdown. See [Configuring a provider](#configuring-a-provider).
- **API key in the OS keyring** — never in the settings file, never sent back to the page. See [Where the key lives](#where-the-key-lives).
- **Desktop shell** — a sidebar that replaces the bottom bar past 720px, cards that use the width (status and jobs side by side, file list beside the preview), a job panel in the corner instead of a sheet across the bottom, hover/focus states, and keyboard shortcuts. Under 720px it is the phone layout: one column, an icon nav along the bottom, the job card as a bottom sheet, a Save bar that stays in reach on Settings, and the Android back button/gesture closing the topmost dialog, sheet or view before it leaves the app. On Android the system bars are drawn dark to match, and the page is padded for the status bar, gesture bar and keyboard so a focused field is never hidden.
- **Sideload builds** — Windows installer (NSIS `.exe`), desktop binary, `.deb`, and a signed Android APK.

Not here yet: Android Keystore-backed encryption for the key and tokens (Android uses app-private storage), iOS, and the world-pack UI (the world skills run, the board slot is there, but there is no world browser).

## Requirements

**Windows desktop**

- Rust (MSVC toolchain, pinned in `rust-toolchain.toml`) and the *Desktop development with C++* workload from Visual Studio Build Tools.
- WebView2 — already part of Windows 11.
- The tauri CLI for release builds: `cargo install tauri-cli --version "^2" --locked`.
- Smart App Control blocks `rustc.exe` and every freshly built test binary (`os error 4551`). Turn it off in *Windows Security → App & browser control* to build here.

**Linux desktop**

- Rust (the toolchain is pinned in `rust-toolchain.toml`) with `rustfmt` and `clippy`.
- Runtime: `libwebkit2gtk-4.1-0`, `libgtk-3-0`.
- To *build*: the dev packages — `libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev`.

If `libwebkit2gtk-4.1-dev` is not installable on your distro, unpack its headers into a prefix and point pkg-config at it:

```bash
PKG_CONFIG_PATH=$HOME/.local/tauri-linux-dev/usr/lib/x86_64-linux-gnu/pkgconfig \
  cargo build -p storycraft-app
```

The runtime `.so` is enough to *run*; only linking needs the `-dev` package.

**Android APK**

- JDK 17+ (`JAVA_HOME`).
- Android SDK with `platform-tools`, `build-tools`, a platform, and an NDK (`ANDROID_HOME`, `NDK_HOME`).
- The `aarch64-linux-android` Rust target (`rustup target add aarch64-linux-android`).
- The Gradle build writes to `~/.gradle` and tauri's plugin build scripts write into `$CARGO_HOME/registry`, so both must be writable.

## Build and run

```bash
# desktop, dev
cargo run -p storycraft-app

# desktop, release binary -> target/release/open-storycraft
cd crates/storycraft-app && cargo tauri build --no-bundle

# desktop, installable package
cd crates/storycraft-app && cargo tauri build --bundles nsis   # Windows -> target/release/bundle/nsis/*.exe
cd crates/storycraft-app && cargo tauri build --bundles deb    # Linux
```

Build releases through the tauri CLI. It adds the `tauri/custom-protocol` feature, which is what a release build is meant to have — the equivalent plain cargo command is `cargo build --release -p storycraft-app --features tauri/custom-protocol`.

The front end is not bundled or transpiled — `ui/index.html`, `ui/app.css`, `ui/app.js` are loaded as they are, and they are compiled into the binary at build time. Edit them, then rebuild.

### Where the skills come from

Resolution order (`crates/storycraft-app/src/paths.rs`):

1. `skills_dir` from settings, if set.
2. On Android, the pack embedded in the library at build time and extracted into app storage on first use — APK assets are not files, so `std::fs` cannot read them where Tauri points resources. The extraction carries a version marker, so installing a new app version replaces it and skills dropped from the pack do not linger.
3. The bundled resources — `<resource dir>/skills/fiction-genre/SKILL.md`, which is how the `.deb` ships the pack.
4. A walk up from the working directory for `open-storycraft/` or `skills/`.

So a release binary run from the repo root finds the vendored pack, and `STORYCRAFT_SKILLS=/path/to/pack` overrides everything.

## Configuring a provider

Settings live in the app config directory — `%APPDATA%\dev.openstorycraft.app\app.json` on Windows, `~/.config/dev.openstorycraft.app/app.json` on Linux (mode `0600`). Override the path with `STORYCRAFT_APP_SETTINGS=/tmp/app.json` (useful for throwaway runs).

| Field | Notes |
|---|---|
| Provider | `openai-compat`, `xai-apikey`, or `grok-oauth` (community OAuth, needs `Sign in`) |
| Base URL | The base **including** `/v1`, e.g. `https://api.x.ai/v1`, `http://127.0.0.1:1234/v1` |
| API key | Bearer token, write-only in the form. Stored in the OS secret store, never in the settings file — see [Where the key lives](#where-the-key-lives) |
| API style | `chat_completions` or `responses` (xAI prefers Responses) |
| Model | Type an id, or pick one from the provider |
| Cheap model | Optional. Editorial and kill-pass skills use it when set |
| Per-skill models | `kill-flat=mini` per line, overrides both of the above |
| Token budget | Packer character budget. `0` uses the core default |
| Skills dir | Override for the vendored pack |
| Overlays | `ao3-*` / `anti-slop-editor`. Off unless enabled, and only if the folders exist |

### The model picker

Fill in the URL and key, then press **Load models from the API**. The app calls `GET {base_url}/models` with your bearer token and fills both dropdowns — *Pick a model* and *Pick the cheap model*. Choosing an entry writes into the text field next to it; the text field stays the source of truth, so a hand-typed id is kept and labelled `not in the provider list`. Saving does not require a successful listing.

Details worth knowing:

- A server that only answers under the versioned path is retried once at `{base_url}/v1/models`, so pasting a bare host URL still works.
- The listing is sorted and de-duplicated, and understands `{"data":[{"id":…}]}`, `{"models":[…]}` (with `id` or `name`), and a bare array.
- Failures are reported inline under the button, not as a toast: a bad key says `provider rejected credentials`, a dead host says `HTTP transport error`, an unrecognised body says `provider returned no model list; type the model id instead`.
- A failed refresh keeps the last good list, so options you already loaded are never thrown away.

```bash
# what a provider serves, without the app
curl -s -H "Authorization: Bearer $STORYCRAFT_API_KEY" https://api.x.ai/v1/models | head -c 400
```

### Connection profiles

Settings → Connection holds any number of named profiles. Each one keeps its own provider type, base URL, API style, model, cheap model, per-skill models, and API key. The selected profile is the one runs use; pick another and press **Save settings** to switch.

- **New profile** starts an empty one; **Delete profile** removes it *and its stored key* when you save. The last profile cannot be deleted.
- Keys are per profile: the first profile keeps the Credential Manager entry `api-key`, later ones get `api-key/<profile id>`. Typing a key always stores it for the profile on screen.
- A settings file from an older build (one provider at the top level) loads as a single profile called **Default**, with its key where it already was.
- Token budget, skills folder and overlays are global, not per profile.

### Where the key lives

Not in the settings file. The settings hold everything else — URL, provider, model, budget — and none of it is secret. The API key goes to the OS secret store, and the app tells you which one it got:

| Platform | Store |
|---|---|
| Linux | Secret Service (gnome-keyring, KWallet) |
| macOS | Keychain |
| Windows | Credential Manager |
| Desktop with no keyring reachable (container, headless, minimal session) | `api-key` in the app config directory (`%APPDATA%\dev.openstorycraft.app\` on Windows, `~/.config/dev.openstorycraft.app/` mode `0600` on Linux), and the settings screen says so instead of pretending |
| Android | App-private storage inside the OS app sandbox — this build has no Android keyring backend |

Details:

- The key is **write-only** in the UI. The host never sends it back, so it cannot appear in the DOM, a devtools session, a screenshot, or a support paste. The field starts empty and only ever carries a new value up. `get_settings` reports `has_api_key` and `secret_backend`, never the key.
- Saving an empty field leaves the stored key alone. **Forget stored key** marks it for removal and the next *Save settings* applies it.
- A key typed into the form is used by **Load models from the API** immediately, before you save it — so you can test a key without storing it.
- Upgrading from an older build: if `app.json` still has a plaintext `api_key`, the first launch lifts it into the secret store and rewrites the settings without it. A key already in the store wins, so a stale file cannot clobber a newer key.
- `STORYCRAFT_SECRET_BACKEND=keyring|file` forces a backend. Useful on a headless server (`file`) or to prove the keyring path is really being used (`keyring`).
- The value prints as `[redacted]` in every `Debug`/`Display` path and wipes its bytes when dropped.
- The xAI OAuth tokens go to the same store (account `oauth-tokens`). Windows Credential Manager caps a secret at 2560 bytes; a token set bigger than that falls back to `oauth.json` in the config directory, with a warning in the log. Tokens an older build left in `oauth.json` are still read, and move into the keyring on the next refresh.

Other hardening on the same pass: the webview runs under a CSP that allows only same-origin scripts and styles (inline styles excepted, since the UI sets a few) and only IPC connections, with `object-src`, `base-uri`, `form-action` and `frame-ancestors` denied; plugin permissions are limited to the dialog, opener and notification plugins the UI actually uses; every path the webview hands back is validated before it is joined or written.

## Storybible

A storybible is one markdown file that carries a whole book. Two skills, a pair:

| Skill | Does | Runs |
|---|---|---|
| `fiction-storybible` | Writes `storybible.md` beside `Wiki/` — assembling existing canon verbatim, or authoring a new book | model call |
| `storybible-import` | Splits that file into `Wiki/…` and `Chapters/…` | on device, no key |

Each document in the file is a `---` frontmatter block naming where it lands (`path:` for an exact destination, `slot:` for a canonical one), then the body:

```markdown
---
path: Wiki/Style/genre.md
working_title: "Salt Ledger"
genre: Mystery
---

# Genre
```

The routing keys are stripped on the way out, so every written file keeps only its own frontmatter — which means the file round-trips. The full format, including what each destination must contain, is [open-storycraft/fiction-storybible/references/storybible-format.md](open-storycraft/fiction-storybible/references/storybible-format.md).

In the app: **Book → Storybible**. `Write` drafts a bible from the book; `Import` unpacks one. `Import` stays disabled until a `storybible.md` exists, and the status board routes a folder that has a bible but no `Wiki/` straight to `Next: storybible-import`.

A complete sample lives at [crates/storycraft-core/tests/fixtures/bible-only/storybible.md](crates/storycraft-core/tests/fixtures/bible-only/storybible.md). To try it from the CLI:

```bash
mkdir -p /tmp/books/salt-ledger && cp "crates/storycraft-core/tests/fixtures/bible-only/storybible.md" /tmp/books/salt-ledger/
storycraft status /tmp/books                    # Next: storybible-import
storycraft run storybible-import /tmp/books --commit
storycraft status /tmp/books/salt-ledger        # Now: Next: fiction-writechapter
```

## CLI

`cargo run -p storycraft-cli --` (binary name `storycraft`) drives the same core without the GUI.

```bash
storycraft status [PATH]                  # the board, as text
storycraft next [PATH]                    # just the next skill and why
storycraft skills                         # the vendored pack, frontmatter only
storycraft pack <skill>                   # files a run would pack, not the whole folder
storycraft run <skill> [PATH] [--commit]  # one skill; preview unless --commit
storycraft jobs|save|reject|diff [PATH]   # job plumbing
storycraft export [PATH]                  # zip Wiki/ and Chapters/
storycraft auth login|status|logout       # xAI community OAuth
storycraft namegen|town|burstiness        # local tools, no model call
```

`run` takes the provider as flags: `--base-url`, `--api-key` (or `STORYCRAFT_API_KEY`), `--model`, `--cheap-model`, `--route skill=model`, `--api-style`, `--budget`.

## Repository layout

```
crates/storycraft-core   Wiki discovery, status board, packing, jobs, storybible format
crates/storycraft-llm    OpenAI-compatible streaming client (chat_completions, responses)
crates/storycraft-auth   Device-code OAuth and token storage
crates/storycraft-tools  On-device tools: Markov names/towns, burstiness
crates/storycraft-cli    `storycraft` binary
crates/storycraft-app    Tauri 2 shell: commands, host loop, settings, window sizing
ui/                      The whole front end (HTML, CSS, JS; no bundler)
open-storycraft/         The vendored skill pack (see its own README)
book/                    Working fixture used during development
docs/adr/                Architecture decisions
```

## Tests and checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo doc --workspace --no-deps      # broken doc links are denied
```

Fixtures under `crates/storycraft-core/tests/fixtures/` include a planned book (`planning-done`) and a bible-only folder (`bible-only`).

## Keyboard

| Keys | Does |
|---|---|
| `Ctrl+1` … `Ctrl+5` | Home / Book / Write / Edit / Settings |
| `Ctrl+R` | Re-read the board from disk |
| `Ctrl+Enter` | Run the recommended next skill |
| `Ctrl+K` | Run any skill (command palette) |
| `Ctrl+O` | Open a book folder |
| `Ctrl+S` | Save the preview into the Wiki |
| `Esc` | Close the job card |

They are listed in Settings → Shortcuts, and the buttons they mirror carry the hint in their tooltip.

The shell is responsive rather than fixed: past 720px the nav becomes a left sidebar and the views lay their cards out in columns; under it you get the phone layout, which is what the Android build always sees. The desktop window opens at 980×700 and is clamped to the monitor work area at startup (`crates/storycraft-app/src/window.rs`).

## Building the APK

On Windows, install Android Studio and, from its SDK Manager, an SDK platform, *NDK (Side by side)*, and *Android SDK Command-line Tools*. Android Studio bundles a JDK, so there is nothing else to install:

```powershell
$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:NDK_HOME = (Get-ChildItem "$env:ANDROID_HOME\ndk" | Select-Object -Last 1).FullName
rustup target add aarch64-linux-android x86_64-linux-android

cd crates\storycraft-app
cargo tauri android build --apk --target aarch64
& "$env:ANDROID_HOME\platform-tools\adb.exe" install -r gen\android\app\build\outputs\apk\universal\release\app-universal-release.apk
```

After a fresh `cargo tauri android init`, run `android-overlay\apply.ps1` to copy the Kotlin helpers back in.

On Linux:

```bash
export JAVA_HOME=~/path/to/jdk17
export ANDROID_HOME=~/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/<version>

cd crates/storycraft-app
cargo tauri android build --apk --target aarch64     # arm64 phones
cargo tauri android build --apk --target x86_64      # emulator
adb install -r gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

A release APK must be signed. Gradle reads `gen/android/keystore.properties` (gitignored), which points at a keystore:

```bash
cd crates/storycraft-app/gen/android
keytool -genkeypair -v -keystore upload-keystore.jks -keyalg RSA -keysize 2048 \
  -validity 10000 -alias upload
printf 'keyAlias=upload\npassword=<password>\nstoreFile=../upload-keystore.jks\n' > keystore.properties
```

Keep the keystore and its password: Android refuses to install an update signed with a different key. A debug APK (`--debug`) needs no keystore but is roughly twenty times larger and unoptimized. `storeFile` is resolved relative to `gen/android/app/`, which is why the example writes `../upload-keystore.jks`.

## Releases

Prebuilt binaries are on the [Releases page](../../releases): Windows installer and portable zip, Linux `.deb`, AppImage and tarball, and an Android APK.

They are built by `.github/workflows/release.yml`. Push a tag (`git tag v0.1.0 && git push origin v0.1.0`) or run the *Release* workflow by hand with a tag name; it builds on GitHub's Windows and Ubuntu runners and attaches everything to a release for that tag.

To sign the APK with your own key (needed for each release to install as an update over the last), add repository secrets `ANDROID_KEYSTORE_BASE64` (`base64 -w0 upload-keystore.jks`), `ANDROID_KEYSTORE_PASSWORD` and optionally `ANDROID_KEY_ALIAS` (default `upload`). Without them each run signs with a throwaway key.

## Known limitations

- The bundle identifier `dev.openstorycraft.app` ends in `.app`; tauri warns about it. Harmless on Android, Windows and Linux, but changing it later changes the Android package name and the desktop config directory.
- A local `--target aarch64` build ships `arm64-v8a` only; the release workflow builds `arm64-v8a`, `armeabi-v7a` and `x86_64`.
