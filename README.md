# Poe2Launcher

Desktop launcher for the Path of Exile 2 private server (Poe2ModdingProject). It supports every client build that has a `versions/<version>.toml` in Poe2Protocol (today 0.5.5e.hotfix1) plus an *Official (read-only)* mode, and does for each player what `cargo xtask deploy-client` and `run-client` do for the developer.

The launcher has no login of its own and creates no accounts. Accounts belong to each server; the player signs in on the game's login screen. It manages several servers: ones you join (My servers, the public list) and ones you host on this computer.

Tauri 2 (Rust backend in `src-tauri/`) with a Svelte 5 + Vite + TypeScript UI (`src/`), game-styled dark theme (light parchment in Settings) from `src/tokens.css` and `src/lib/motion.ts` (identical copies in the web admin), fades and slides that follow `prefers-reduced-motion`. UI preview without the backend: `npm run dev`, then `http://localhost:1420/?mock` (`&versions`, `&official`, `&empty`, `&fresh`, `&mods`, `&modsync[=off|auto]`, `&theme=light`, `&installing`, `&paused`, `&lowspace`, `&offline`, `&nonews`, `&motd`, `&nolist`, `&nobackups`, `&nologs`; the mock is `src/lib/mock.ts`, not in release builds).

The user guide (set-up, first run, hosting, server updates, client hook, releasing, troubleshooting) is [docs/guides/launcher.md](https://github.com/WaystoneWorks/Poe2ModdingProject/blob/main/docs/guides/launcher.md); this file is the developer's view.

## Pages

Sidebar: the **game version picker** (*Official (read-only)* first, then every embedded private version, saved as `game_version` in `settings.json`; refused while an install, the game or the hosted server runs), Play, Servers, News, Library, Settings, the update badge and About.

| Page | Does |
|---|---|
| Play (`#/home`) | Selected server with live status and ping, the Play button, the *Get ready to play* checklist, newest news, server-update notice, the server's `motd` (`GET /api/v1/server`). Private version: deploys the hook (with the server's register page for the game's Create account button) and starts the game at its own login screen. Official: one-time risk confirmation, then the read-only deploy (`mode = "official"`, empty `trusted_servers`, original gateway list) and the game |
| News | Release notes of Poe2Launcher, Poe2Server (`server-v*`) and Poe2ClientHook (`hook-v*`) from the GitHub releases API plus `news.json` of the hub repository; cached, quiet offline |
| Servers | *My servers*: profiles (name and web API address), *Check* (`GET /server`), *Web admin* and *Create account* open the server's pages in the browser; servers reporting another client version are marked *incompatible* / *needs client X*; plain `http://` is marked. *Browse*: the public list (`WaystoneWorks/Poe2ServerList`, `servers.json`), live status and ping, filter/search/sort, *Add to my servers*. *Host*: servers on this computer (private versions only), see the guide |
| Library | *Game*: Steam depot download with the player's own Steam login (pinned manifests for a private version, current build for Official), pause/resume, disk-space check, verify/repair, **Client hook** card. *Mods*: mod cache per server and mod browser. *Logs*: game and server logs, crash dumps, *Copy for a bug report* (redacted) |
| Settings | per-version install dir, hook DLL dir, Proton (Linux), **Steam account** (*Log out of Steam*), Steam download options, theme, window frame, *Minimize the launcher while the game is running*, desktop entry (Linux), *Mod sync*, launcher updates |

Hosting (own folder, ports, config and backups per server; shared packages): download the matching `poe-server` release (SHA-256 checked, Poe2ModdingProject releases as fallback, both package layouts) or pick a local package; one-click server updates with rollback; start/stop with streamed log; usage; optional autostart; restart supervision (exit code 75 from the web admin's Restart starts it again); database backups (online SQLite backup, restore with a safety backup, automatic backup before each start, keep the last N). The first admin is made in the web admin and players register on the server's register page.

## Clone and build

The hook deploy library is a nested submodule (Poe2ClientHook nests Poe2Protocol), so a recursive clone builds with no git tricks:

```sh
git clone --recursive https://github.com/WaystoneWorks/Poe2Launcher.git
cd Poe2Launcher
npm ci
npm run tauri dev                  # development window (Vite on :1420)
npm run tauri build -- --debug     # debug build + deb/rpm/AppImage on Linux
```

Linux build dependencies: `webkit2gtk-4.1`, `gtk3`, `libsoup3`, `dbus`, `librsvg` (Arch: `webkit2gtk-4.1 gtk3 libsoup3 librsvg`; Debian: `libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libdbus-1-dev librsvg2-dev`). Node 22, current stable Rust. In a Poe2ModdingProject checkout the launcher is `modules/launcher`; its `src-tauri` has its own `[workspace]`.

## Tests and CI

```sh
npm run check && npm test                 # svelte-check + tsc, vitest
cd src-tauri && cargo test                # unit tests + tests/{mods_flow,version_switch,core_flow,web_admin,host_flow,host_update,hook_flow}.rs (mock servers, mock keyring, fake Proton, fake packages)
cargo test --test fetch_tool -- --ignored          # downloads the pinned DepotDownloader, runs `-V` (never logs in)
cargo test --test host_flow real_package -- --ignored --nocapture   # a REAL server package (needs POE2_TEST_SERVER_PACKAGE and POE2_TEST_GAME_DIR, see the test's comment)
cargo test --test hook_flow real_release -- --ignored --nocapture   # the REAL hook release from GitHub (network; needs POE2_TEST_HOOK_REAL=1)
cargo test --test core_flow real_keyring -- --ignored   # needs an unlocked system keyring
POE_LAUNCHER_TEST_URL=http://127.0.0.1:18080 cargo test --test core_flow real_server -- --nocapture   # against a real poe-server
```

GitHub Actions (`.github/workflows/ci.yml`): Ubuntu (WebKitGTK dev packages, `npm run check/test/build`, `cargo fmt --check`, `clippy -D warnings`, `cargo test`) and Windows (`cargo check`); submodules fetched with `submodules: recursive`.

## Layout

```
src/                 Svelte UI: App.svelte (shell, sidebar), pages/ (Home = Play, Servers, News, Library (Install = Game, Mods, Logs), Settings, About), lib/ (ipc.ts, app.svelte.ts (state), gameversion.ts, install.ts, transfer.ts, markdown.ts (safe subset), browse.ts, logview.ts, rate.svelte.ts, mods.ts, update.ts, frame.ts, motion.ts, mock.ts, hook.ts), ui/ (HostPanel, Drawer, VersionPicker, ServerBrowser, HookPanel, SetupChecklist, UpdatePanel, ResizeHandles, ...)), tokens.css, app.css, assets/
assets/icon.svg      app icon source; `npx tauri icon assets/icon.svg` regenerates src-tauri/icons/
src-tauri/src/
  core.rs            service layer (everything the UI can do; no Tauri types, testable)
  commands.rs        thin #[tauri::command] wrappers; lib.rs wires events to the window
  api.rs             web API client (reqwest, rustls), incl. GET /me and POST /web-token
  host.rs            hosting, pure parts: per-server settings (servers/<id>.json), release lookup, checksum, package check and extraction, server.toml from server.example.toml (+ `carry_over_config`), LAN/public IP, port check
  host_run.rs        the poe-server process: start/stop, log ring buffer, one-shot commands
  host_ops.rs        hosted servers on Core: create/delete, install, apply settings, start/stop/status/usage
  host_update.rs     server updates: quiet check, one-click update with rollback
  backup.rs, backup_ops.rs   database backups (SQLite online backup, list, restore, prune, backup.json) and the automatic backup before a start
  hook.rs            client hook, pure parts: release lookup (tag `hook-v*`), asset names, `release.json`, per-file SHA-256 check, install into `<data>/hook/packages/<version>`
  hook_ops.rs        client hook on Core: status, update check, download + verify + install, `hook_dll_dir`, check before each deploy
  news.rs            news: GitHub release notes + news.json, validation, cache
  serverlist.rs      public server list (Poe2ServerList): fetch, validation, cache, parallel probes of GET /server
  logs.rs            log and crash viewer, bug-report text and redaction
  space.rs           disk-space check, depot weights
  profiles.rs        server profiles (profiles.json, no secrets)
  keystore.rs        TokenStore: KeyringStore (keyring crate) / MemoryStore (tests)
  settings.rs        settings.json
  version.rs         versions/<version>.toml (embedded by build.rs); VersionFile::resolve, VersionFile::official() (no pinned manifests)
  steam.rs           the remembered Steam login (account.config of DepotDownloader): status and log out
  desktop.rs         Linux .desktop entry + icons for the AppImage
  depot.rs           DepotDownloader: pinned fetch, argument builder, prompt/progress parser, runner
  verify.rs          install verification
  play.rs            deploy options, Proton/native launch (poe-hook-deploy)
  mods.rs            compatibility (game version, hook ABI), load-order resolver, server-half paths
  modcache.rs, modsets.rs, modsync.rs, modindex.rs, mod_ops.rs   mod cache, per-profile sets (apply_to_game), server mod sync, mod browser index, Core operations (prepare_mods at Play)
  paths.rs           launcher dirs and the "never the Steam install" guard
versions/
  <version>.toml     COPY of Poe2Protocol's versions/<version>.toml (modules/protocol/versions/ in the hub; refreshed by `cargo xtask versions`, never edited here): app, depots + manifests (install order), [exe] and [verify]; other sections ignored
  depotdownloader.toml  pinned DepotDownloader release, per-platform SHA-256
client-hook/         submodule: Poe2ClientHook (poe-hook-deploy as a path dependency)
```

## Data and secrets

| What | Where |
|---|---|
| App identifier (`org.poe2modding.launcher`) | Wayland `app_id` / WM class (`glib::set_prgname` in `lib.rs`) and the webview storage folder |
| Profiles, settings | `<config>/poe2-launcher/profiles.json`, `settings.json` (`~/.config/...`, `%APPDATA%\...`) |
| DepotDownloader, games, Proton prefixes, logs | `<data>/poe2-launcher/{tools,game/<version>,game/official,proton-prefix,proton-prefix-official,logs,crash-dumps}` |
| Remembered Steam login (DepotDownloader's `account.config`, a token, never the password) | `<data>/poe2-launcher/steam-login/IsolatedStorage/...` on Linux; `%LOCALAPPDATA%\IsolatedStorage` on Windows. *Log out of Steam* deletes it |
| `.desktop` entry and icons (only after *Install desktop entry*) | `~/.local/share/applications/org.poe2modding.launcher.desktop`, `~/.local/share/icons/hicolor/*/apps/org.poe2modding.launcher.*` |
| Hosted servers' settings (no secrets) | `<config>/poe2-launcher/servers/<id>.json` (a launcher before 0.6.0 kept one in `host.json`; it moves to `s1`); backup choices in `servers/<id>.backup.json` |
| News and server-list cache (safe to delete) | `<data>/poe2-launcher/cache/{news.json,serverlist.json}` |
| Server packages (shared) | `<data>/poe2-launcher/server/{packages,downloads}` |
| A hosted server's database, config, mods, log, backups | `<data>/poe2-launcher/servers/<id>/{data,server.toml,mods,config-backups,server.log,backups}` (outside the package, so an update keeps accounts and characters) |
| Steam password | typed in the UI, written once to DepotDownloader's stdin, wiped. Not stored, not on a command line |

`POE2_LAUNCHER_HOME=<dir>` puts config and data under `<dir>/config` and `<dir>/data` (tests, portable use).

**Never the Steam install.** The launcher does not read or write `steamapps/common/Path of Exile 2` or `compatdata/2694490`. Install, DLL and similar directories inside a Steam library are refused (`paths.rs`); the Proton prefix is its own. On Linux it only reads Proton itself from `<steam root>/steamapps/common/<Proton>` to start it.

## Game versions and install

The dropdown lists every embedded `versions/<version>.toml` (adding a build is adding a file, see [new-game-version.md](https://github.com/WaystoneWorks/Poe2ModdingProject/blob/main/docs/guides/new-game-version.md)) plus *Official (read-only)*, which reuses the newest file's depots without manifests, with its own folder `game/official` and the hook in read-only mode. The selection is `Settings.game_version`; install folders are per version (`Settings.install_dirs`; the old single `install_dir` moves to the oldest version when read).

For 0.5.5e.hotfix1: app 2694490; depot 2694491 manifest 4712263798014036045 (content), depot 2694492 manifest 4973272861086296374 (Windows exe). Both go into one directory in file order (exe over content), in **one** DepotDownloader run so Steam Guard is asked once.

- DepotDownloader **3.4.0**, self-contained (no .NET), pinned with SHA-256 in `versions/depotdownloader.toml`; downloaded over HTTPS to `<data>/tools/`, refused and deleted on a mismatch.
- Arguments: `-app 2694490 -depot A -manifest MA -depot B -manifest MB -username U [-remember-password] [-no-mobile] -dir DIR [-validate]`. `-remember-password` only if ticked; never `-password`.
- The runner reads raw chunks (prompts have no line end), recognises `Enter account password for ...` (answered from the form) and `STEAM GUARD! ...` (the UI asks, the code goes to stdin), parses `NN.NN% file` progress and `Downloading depot N`, and reports failures (`Failed to authenticate...`).
- Verification (`verify.rs`): file count and size floors, key files with exact sizes, SHA-256 of `PathOfExileSteam.exe`. The reference numbers were measured on the GameFiles copy, not a launcher download: unconfirmed until a first real install (a checksum mismatch is a warning until `[exe] sha256_enforce = true`).

## Play

1. `GET /server` gives the login and instance addresses (`login_host`, `login_port`, `instance_port`, `gateway_region`).
2. `poe_hook_deploy::deploy`: `gateway_list.txt` with one entry, `version.dll` + `poe_hook_core.dll` from the configured DLL directory, `poe-hook.toml` with `mode = "private"`, `login_mode = "standalone"`, `trusted_servers` = the server's login and instance `host:port`, `steam_appid.txt`.
3. Launch: Linux `proton run PathOfExileSteam.exe --success-event 0` through `ProtonLaunch` (launcher-owned prefix, created with `proton run wineboot -u` on first use; Steam must be running). Windows: `PathOfExileSteam.exe --success-event 0` directly (not testable on Linux CI). Game output goes to `<data>/logs/game-<time>.log`.

Official (read-only) is the one other deploy: `play::official_deploy_options` writes exactly what `cargo xtask deploy-client --official` does and never a trusted server or gateway (tests in `play.rs` and `tests/version_switch.rs`).

**Hook DLLs** (`hook.rs`, `hook_ops.rs`, `ui/HookPanel.svelte`, `ui/SetupChecklist.svelte`, `lib/hook.ts`) come from, in order: the *Hook DLL directory* setting (a Poe2ClientHook build, e.g. `modules/client-hook/target/x86_64-pc-windows-msvc/release`), the **client hook downloaded by the launcher**, or the old default folder `<data>/hook-dlls`. The download is the newest stable `WaystoneWorks/Poe2ClientHook` release (tag `hook-v*`, asset `poe-hook-v<ver>-game-<games>-windows-x86_64.zip` + `.sha256`) whose name lists the selected game version (Official uses the newest private version's). The archive is checked against the `.sha256`, unpacked with the server-package guards, every file is checked against `release.json` (an unlisted file refuses the package), and the package must support the game version. It lives in `<data>/hook/packages/<hook version>/` and is re-checked before each deploy. Without any hook Play is blocked. See the guide and [releasing-hook.md](https://github.com/WaystoneWorks/Poe2ModdingProject/blob/main/docs/guides/releasing-hook.md).

## Mods

Spec: [mod-manifest.md](https://github.com/WaystoneWorks/Poe2ClientHook/blob/main/docs/mod-manifest.md) (`modules/client-hook/docs/mod-manifest.md`); the manifest (v1, v2) is parsed by the shared `poe-hook-mods` crate (the nested `client-hook/` checkout). A mod has a **side**: `client`, `server` or `both`. The launcher's store holds the client half only: a server-only mod is refused, and a `both` package is unpacked without its `server/` half.

- **Cache.** `<data>/mods/cache/<id>/<version>/` holds every verified version (`mod/` = unpacked client half, `meta.json` = SHA-256, size, source). A package enters only through `poe_hook_mods::verify_package` / `verify_package_bytes` (hash, `<id>-<version>.zip` name, paths, links, limits, CRCs, manifest); unpacking happens in a scratch folder renamed into place at the end (`modcache.rs`).
- **Per-server sets.** A profile (`profiles.json`, `mods`) stores its enabled mods (id + exact version); `<config>/modsets.json` holds the ones enabled for all servers. At Play the set is the server's required mods (always on) + the profile's + the global ones, dependencies pulled from the cache, conflicts/incompatible mods dropped with a note, ordered by dependencies and `after/before` (`modsets.rs`). `apply_to_game` copies it to `<install>/poe-hook-mods/<id>/` (a marker file says the launcher put it there; other folders are never touched), writes `state.toml` and removes what the previous server needed. Official has no mods.
- **Mod sync** (`modsync.rs`, `mod_ops.rs`). `GET /api/v1/server` may carry `mods` (an array, or `{"client_mods": [...]}`); each entry: `id`, `version`, `side` (`client`/`both`; `server` entries ignored), `sha256` (64 hex), `size`, optional `required` (default true), `hook_api`, `game_versions`, `sources` (`{"type":"server","path":"/api/v1/mods/<id>/<version>.zip"}` and/or `{"type":"url","url":"https://..."}`; with none, the default server path), `name`, `authors`, `description`, optional `code` (default: has a `hook_api`). The launcher compares with the cache (ready / missing / other build / incompatible / no source) and acts by the **Mod sync** setting (`mod_sync`): `off` shows what is missing and blocks Play, `ask` (default) shows a dialog with Download, `auto` downloads, verifies and switches silently, code mods included. A download is kept only if its SHA-256 equals the advertised one and `verify_package` accepts it; size is capped at the advertised size. Plain `http://` to another machine needs the profile's confirmation (the server's path is then not used); extra URLs must be `https://` (or this computer). Servers without `mods` work unchanged; an unreadable `mods` field counts as none.
- **Mod browser** (`modindex.rs`, Library > Mods > Browse): `index.json` of `WaystoneWorks/Poe2Mods` (schema in `docs/mod-index-template/` of Poe2ModdingProject), validated per entry, cached 10 min (`<data>/cache/modindex.json`), saved copy offline; filter by text, side, tag and game version; install into the cache (hash from the index) and enable for one server or all.
- **Settings**: *Mod sync* (Off / Ask / Auto), cache size, *Clear unused cached mods*.

## Window, icon and Wayland

`tauri.conf.json`: `decorations: false`, `visible: false` (shown from `setup` after the frame choice and window icon), square window (a transparent one rendered only partly under WebKitGTK/Wayland), capabilities for minimize, maximize, close, start-dragging, start-resize-dragging and set-decorations. `data-tauri-drag-region` marks the draggable title bar; resize handles are `ui/ResizeHandles.svelte`. `Settings.custom_frame = false` switches to the system frame. On Linux `lib.rs` sets GTK's program name to the identifier before GTK starts (the Wayland `app_id`); the window icon comes from the `.desktop` file (`desktop-template.desktop` sets `StartupWMClass=org.poe2modding.launcher` and `Categories=Game;`).

## Self-update and releases

The launcher updates itself with `tauri-plugin-updater` (+ `tauri-plugin-process` for the restart; capabilities `updater:default`, `process:allow-restart`). `tauri.conf.json` has `bundle.createUpdaterArtifacts = true` and `plugins.updater`: endpoint `https://github.com/WaystoneWorks/Poe2Launcher/releases/latest/download/latest.json`, the **public** signing key (`pubkey`), Windows install mode `passive`. UI: `src/lib/update.ts` (tested with a fake plugin), `update.svelte.ts`, `ui/UpdatePanel.svelte`, and the badge in the top bar. A failed check (offline, 404) is the quiet "no update info" state; a failed download or signature check is an error and installs nothing. Linux updates replace the AppImage in place (only when running as an AppImage); Windows runs the NSIS installer.

`.github/workflows/release.yml` builds on a pushed tag `v*`: a draft release, then the AppImage (`ubuntu-22.04`, `tauri build --bundles appimage`) and the NSIS installer (`windows-latest`, `--bundles nsis`) with `tauri-apps/tauri-action@v1`, signed with the secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `latest.json` merged, published at the end. Tagging, secrets and key care: the "Releasing" section of the guide. Local check: `TAURI_SIGNING_PRIVATE_KEY=... TAURI_SIGNING_PRIVATE_KEY_PASSWORD=... npx tauri build --bundles appimage` gives the AppImage and its `.sig`.

## Commands the UI invokes

`app_info`, `profiles_list`, `profile_add`, `profile_update`, `profile_remove`, `profile_confirm_plain_http`, `server_info`, `web_admin_open`, `settings_get`, `settings_set`, `game_version_set`, `steam_login_status`, `steam_logout`, `desktop_entry_status`, `desktop_entry_install`, `desktop_entry_uninstall`, `install_status`, `install_start`, `install_input`, `install_cancel`, `install_verify`, `mods_list`, `mod_install`, `mod_set_enabled`, `mod_move`, `mod_auto_order`, `mod_remove`, `mods_dir`, `mod_cache_status`, `mod_cache_prune`, `mod_sync_check`, `mod_sync_apply`, `mod_index_get`, `mod_index_install`, `play` (profile id optional: Official takes none), `play_state`, for the client hook `hook_status`, `hook_check`, `hook_install_latest`, for hosting `host_status`, `host_find_release`, `host_install_latest`, `host_update_check`, `host_update_apply`, `host_install_local`, `host_lan_ip`, `host_public_ip`, `host_setup`, `host_start`, `host_stop`, `host_log_tail`, `host_set_autostart`, `host_open_guide`. Arguments are camelCase from JS (`profileId`, `overrideCompat`); errors are readable strings.

Events: `install:state` (`fetching_tool`, `downloading`, `verifying`, `done`, `failed`, `cancelled`), `install:log`, `install:progress`, `install:prompt` (Steam Guard), `install:verify`, `play:status` (`idle`, `preparing`, `running`, `exited`), `play:warning`, `hook:progress`, `hook:step`, `hook:installed`, `host:status` (`stopped`, `running`, `stopping`, `exited`), `host:log`, `host:progress`, `host:setup`, `host:installed`.

## Security

- Passwords and tokens: the keyring only (`poe2-launcher`); `Debug` output redacts them; the Steam password never touches a command line or a file; the launcher handles no server account passwords.
- **Plain `http://`** (audit S-03): to an address that is not this computer, the launcher sends nothing (no token for the web admin) until you confirm that address on the Servers page; the confirmation is per address and withdrawn when the address changes. A bare `host:port` gets `https://` unless it is this computer or a local network.
- **Open web admin** (admins only): `GET /me` for the role, `POST /web-token` for a single-use token, then `<server address>/#/token-login?t=<token>` in the system browser (Tauri opener). The link is checked to be exactly that form and is never logged. See [web-api.md](https://github.com/WaystoneWorks/Poe2ModdingProject/blob/main/docs/architecture/web-api.md).
- Server packages are verified against their `.sha256` file; archives are unpacked with path, link and size checks.
- Reporting a vulnerability: [SECURITY.md](SECURITY.md).

## Disclaimer

This is an unofficial, fan-made project for personal and educational use. It is **not affiliated with, endorsed by or connected to Grinding Gear Games**; "Path of Exile" is their trademark. **No game files are distributed** in this repository or by the launcher's own releases: the Install page downloads the game from Steam with *your* Steam account, so you need your own legal copy. Use is at your own risk. The client hook that the launcher deploys can be detected as a modification by anti-cheat software. It is for the private server only: the launcher always deploys private mode, and use on official servers is limited to read-only observation by developers (not supported, not recommended).

Contributing: [CONTRIBUTING.md](https://github.com/WaystoneWorks/Poe2ModdingProject/blob/main/CONTRIBUTING.md) in the main repository (checks here: `npm run check`, `npm test`, `npm run build`, and in `src-tauri` `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`).

## Licence

MIT: [LICENSE](LICENSE). Copyright (c) 2026 WaystoneWorks and contributors.
