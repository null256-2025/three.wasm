Original prompt: [sakura_final_spec.md](sakura_final_spec.md) を基に実装してください。画像のようなテイストがいいな

## Phase Tracker

| Phase | Status | Goal | Exit Criteria | Current Notes | Next Action |
|---|---|---|---|---|---|
| Phase 0 | completed | 見た目と基本挙動の土台を作る | 桜の木・島・岩・有限状態の花びら・UI が動作 | `proctree.js` ベースの木、有限状態、可変花びら数 UI は実装済み | なし |
| Phase 1 | in_progress | JS のまま 3 レイヤー分離する | `Tree` / `Falling` / `Ground` が分離され、`ON_TREE` と `ON_GROUND` が毎フレーム更新から外れる | 現在は単一 `InstancedMesh` のまま。Step 1-4 相当は済み、5-8 が未着手 | `ON_TREE` shader sway 化と `Ground` 静的レイヤー化 |
| Phase 2 | not_started | `FALLING` を Wasm 化する | SoA バッファ、`activateReadyPetals()`、`stepFalling()`、`landedIndices` が動作 | Rust / `wasm-pack` ツールチェーン未導入。JS 版のまま | ツールチェーン導入後に `Cargo.toml` と `src/lib.rs` を作成 |
| Phase 3 | not_started | 転送量と更新範囲を最適化する | `activeIndices` ベースの partial upload、不要な `setMatrixAt()` 排除 | Phase 2 前提のため未着手 | `InstancedBufferGeometry` への移行設計を確定 |
| Phase 4 | planned | 10 万枚目標に向けた追加最適化 | 5 万超で破綻しない、10 万枚に向けた GPU 主体設計に道筋 | 現時点では将来計画 | Phase 3 の測定結果を見て WebGPU / 追加最適化を判断 |

## Current Focus

- 最優先は Phase 1 Step 5-8 にあたる 3 レイヤー分離
- 公開 UI 上限は Phase 2/3 完了まで `24000`
- 木の見た目は `proctree.js` を維持し、最適化対象は花びら更新に限定

## Risks / Blocks

- `cargo` と `wasm-pack` が未導入のため、Wasm 実装とビルド検証に入れない
- 現状は AoS + 全件ループ + `setMatrixAt()` のため、10 万枚は設計上まだ非対応
- 花びら形状はフラグメント discard ベースなので、CPU 側改善後も GPU overdraw が残る

- 2026-04-02: `index.html` を最終仕様ベースで全面再構成。`InstancedMesh` 花びら 2000 枚、`ON_TREE -> FALLING -> ON_GROUND` の有限状態、リスポーンなし、島・木・岩・ルート・背景グラデーションを実装。
- 2026-04-02: 参照画像に寄せて、UI をミニマルな淡いピンク基調へ変更。背景はシェーダー球で柔らかいピンクグラデーションにした。
- 2026-04-02: `window.render_game_to_text` と `window.advanceTime(ms)` を追加。自動テストしやすい固定ステップ更新に変更。
- 2026-04-02: Playwright で確認。初期 1.22s 時点で `onTree=2000`、`advanceTime(3000)` 後は `onTree=1925 / falling=74 / onGround=1`、さらに `advanceTime(17000)` 後は `onTree=1342 / falling=147 / onGround=511` を確認。
- 2026-04-02: スクリーンショット確認で、初期の満開感が弱かったため花びらの向き・密度・構図・配色を再調整。
- 2026-04-02: 花びら数を固定値から可変に変更。右上にレンジ入力・数値入力・プリセットボタンを追加し、`window.setPetalCount(count)` と `?petals=24000` のような URL 指定でも反映できるようにした。
- 2026-04-02: 花びら数変更時は `InstancedMesh` とデータ配列を作り直し、満開状態から再初期化する設計にした。上限は現在 `100000`、500 枚刻み。
- 2026-04-02: Chrome DevTools で確認。`window.setPetalCount(24000)` 直後に `total=24000 / onTree=24000`、その後 `window.advanceTime(3000)` を呼んだ状態でも 24000 枚で遷移が継続することを確認。
- 2026-04-03: `petals` の UI 上限、クランプ上限、最大プリセット値を `100000` に引き上げた。
- 2026-04-03: `sakura_wasm_redesign.md` を追加。状態別 3 レイヤー構成、SoA データ配置、`FALLING` のみ wasm 化する方針、GPU/JS/wasm の責務分離、段階的な移行順序を整理した。
- 2026-04-03: `three.wasm` が実在しない前提へ整理。`sakura_final_spec_wasm.md` を更新し、木は `proctree.js` ではなく軽量ブランチスケルトンで表現し、Wasm は花びらシミュレーション専用に使う方針へ修正した。
- 2026-04-03: `index.html` の木生成を軽量化。`proctree.js` 依存を削除し、低ポリ円柱セグメントの幹・枝と枝先アンカーから桜のシルエットを作る実装へ差し替えた。
- 2026-04-03: Playwright で確認。軽量ツリー化後も `fps 60 | petals 2000` で動作し、`window.setPetalCount(16000)` 後も `onTree=15963 / falling=37 / onGround=0` の状態遷移が継続することを確認。コンソールエラーは 0 件。
- 2026-04-03: 方針修正。木の軽量化は本質的なボトルネック対策ではなかったため、`index.html` の木生成を `proctree.js` ベースへ戻した。見た目はリアル寄りを優先し、最適化対象は花びら更新に限定する。
- 2026-04-03: 未実装の Wasm 前提で `100000` を UI に出していたのは不適切だったため、花びら UI 上限と最大プリセット値を `24000` に引き下げた。10 万枚目標は Phase 2/3 実装後に再開する。
- NOTE: 現在の実装は WebAssembly を使っていない。花びら更新ループは JavaScript のままなので、枚数をさらに上げるならまず CPU コストの計測が必要。
- NOTE: `f` キーによるフルスクリーン切替は実装済み。自動ブラウザ操作では `document.fullscreenElement` を安定確認しづらいため、必要なら実ブラウザで追加確認する。
- 2026-04-03: Implemented the Phase 1 3-layer petal split in `index.html`.
- 2026-04-03: `ON_TREE` now uses a shader-driven instanced layer with GPU sway and per-instance visibility toggles.
- 2026-04-03: `FALLING` now uploads only active falling petals into a dedicated instanced layer each frame.
- 2026-04-03: `ON_GROUND` now writes landed petals once into a separate static-ish ground layer instead of updating all petals forever.
- 2026-04-03: Kept `window.render_game_to_text`, `window.advanceTime(ms)`, `window.setPetalCount(count)`, and fullscreen toggle on `f`.
- 2026-04-03: Added URL sync for `?petals=` when preset buttons or controls change the petal count.
- 2026-04-03: Verified in Playwright at `http://127.0.0.1:3001/index.html` for 2000, 16000, and 24000 petals.
- 2026-04-03: Verified `advanceTime(3000)` changes counts correctly, preset-button UI updates URL/state, and fullscreen toggles on/off with `f`.
- 2026-04-03: Started Phase 2 scaffolding. Added `Cargo.toml`, `src/lib.rs`, Rust `SakuraSim` SoA buffers, and the planned exported API for seeding / activation / falling / pointer access.
- 2026-04-03: Added `npm run build:wasm` and `npm run build:wasm:dev` scripts, and ignored `pkg/` / `target/`.
- 2026-04-03: `cargo` / `wasm-pack` are still missing on this machine, so the new Rust crate has not been compiled or integrated into `index.html` yet.
- TODO: Move the current JS falling simulation state to Wasm SoA buffers from `sakura_final_spec_wasm.md`.
- TODO: Replace full falling-buffer uploads with partial uploads / active range updates in the falling layer.
- 2026-04-03: Pivoted Phase 2 away from `wasm-pack` / `wasm-bindgen`. `src/lib.rs` now exports raw `extern "C"` Wasm functions plus linear-memory pointers so the browser can load `target/wasm32-unknown-unknown/release/sakura_sim.wasm` directly.
- 2026-04-03: `cargo build --target wasm32-unknown-unknown --release` now succeeds on this machine. `package.json` was updated so `npm run build:wasm` / `build:wasm:dev` use plain Cargo instead of `wasm-pack`.
- 2026-04-03: `index.html` now loads the raw Wasm module at startup, seeds blossom positions into Wasm memory, and uses Rust for `activate_ready_petals` and `step_falling`.
- 2026-04-03: JS still owns rendering and layer uploads. `ON_TREE` stays shader-driven, `FALLING` reads active transforms from Wasm memory each frame, and `ON_GROUND` appends landed petals from Wasm-reported indices.
- 2026-04-03: Verified at `http://127.0.0.1:3000/` with Playwright/DevTools: initial load has no console errors, `advanceTime(3000)` moves counts forward, `window.setPetalCount(16000)` and `window.setPetalCount(24000)` both rebuild correctly, URL sync still works, and fullscreen toggles on/off with `f`.
- 2026-04-03: Phase 2 follow-up: `FALLING` and `ON_GROUND` layer uploads now mark only the touched buffer ranges instead of flagging the entire attribute payload every frame. Tree visibility updates also upload only the changed span.
- 2026-04-03: Cleaned `index.html` runtime strings and source structure around HUD/title/hint text. Removed duplicate `refreshHud*` overrides and aligned the HTML text with the runtime text.
- 2026-04-03: Re-verified after the partial-upload change: `npm run build:wasm` succeeds, console errors remain at 0, `advanceTime(3000)` still advances falling/landing, `setPetalCount(16000)` / `24000` rebuild correctly, and fullscreen toggle still works.
- 2026-04-03: Reworked `ON_GROUND` into a queued upload path. Landed petals are first appended to `groundLandingQueue`, then flushed to the ground layer with a per-frame `groundUploadBudget`.
- 2026-04-03: Added `window.setGroundUploadBudget(count)` and exposed `pendingGroundUploads` in `render_game_to_text()` so larger-count experiments can observe backlog growth and drain behavior.
- 2026-04-03: Verified the queue behavior at `24000` petals: with `setGroundUploadBudget(1)` backlog grows (`pendingGroundUploads > 0`), and with `setGroundUploadBudget(8192)` it drains back to 0 while rendering stays stable and console errors remain 0.
- 2026-04-03: Disabled OrbitControls auto-rotate so the camera stays still unless the user drags. This avoids constant motion during review and testing.
- 2026-04-03: Added perf instrumentation to `index.html` and `render_game_to_text()`. The text state now reports averaged CPU timings, GPU render timings via `EXT_disjoint_timer_query_webgl2` when available, per-frame upload KB, and a coarse `bottleneck` hint.
- 2026-04-03: Measured the current 24k phase with DevTools. At full bloom (`treeInstances=24000`, `falling=0`) the dominant cost is GPU render time at roughly `gpuRenderMs ~= 10ms` while CPU step cost is near-zero, so the next hard wall is GPU draw/overdraw rather than Wasm activation.
- 2026-04-03: Measured a mid-flight 24k state (`treeInstances ~= 21.5k`, `falling ~= 1.7k`) and saw `gpu_render` remain the top bucket, with `wasm_step` next (~1ms scale) and `fallingUpload` below that. The old `activate_ready_petals` full scan is no longer a meaningful hotspot after the cursor change.
- 2026-04-03: Replaced `activate_ready_petals` full-count scanning with a sorted `activation_order` + `activation_cursor` in Rust/Wasm. Activation cost now scales with newly-ready petals instead of total petal count.
- 2026-04-03: Reworked `ON_TREE` from a visibility-flag approach to swap-remove compaction. `treeInstances` now shrinks as petals leave the tree, which directly reduces the persistent GPU cost during long runs and is a safer stepping stone toward 100k than restoring the UI cap immediately.
- 2026-04-03: Reworked `groundLandingQueue` from `copyWithin()` shifting to a ring buffer. This removes O(queue length) queue maintenance and makes low-budget drain tests stable even with large backlogs.
- 2026-04-03: Expanded `window.setGroundUploadBudget(count)` to allow larger debug budgets for 100k-phase experiments; the clamp is now `1..65536`.
- 2026-04-03: Re-verified after the 100k-prep pass with Playwright and DevTools: `npm run build:wasm` succeeds, console errors stay at 0, `window.advanceTime(3000)` advances state, `window.setPetalCount(16000)` / `24000` still rebuild correctly, fullscreen still toggles on/off with `f`, and the new queue/perf stats are visible in both HUD and `render_game_to_text()`.
- 2026-04-03: Verified the ring-queue drain path specifically at `24000`: with `setGroundUploadBudget(1)` the backlog grows (`pendingGroundUploads=1404` in one sample), and with `setGroundUploadBudget(65536)` it drains back to 0 without console errors.
- TODO: Consider a deeper partial-upload pass that avoids rewriting the whole active prefix when slot order changes under high churn.
- TODO: For the 100k phase, the next likely win is GPU-side work reduction, not more Wasm micro-optimization. Candidate directions are reducing petal overdraw, shrinking on-ground draw cost as density rises, or moving falling transform reconstruction closer to the GPU.
- TODO: If 50k+ tests show `fallingUploadMs` rising faster than expected, prototype a slot-stable falling pool or a GPU reconstruction path before reopening the UI cap beyond 24000.
- 2026-04-03: Reopened the petal UI cap to `100000` and added `50000` / `100000` presets. The HUD hint now explains that very high counts switch to a lighter rendering path automatically.
- 2026-04-03: Added count-based quality profiles in `index.html`. `standard` stays on the original transparent cutout petals, while `dense` / `ultra` reduce renderer pixel ratio, switch petals to a simplified opaque kite mesh, and raise the default ground upload budget.
- 2026-04-03: `render_game_to_text()` now reports `qualityMode` and `rendererPixelRatio`, and the HUD shows the active mode so 100k experiments can confirm whether the app is in the expected render path.
- 2026-04-03: Verified with DevTools at `?petals=100000`. Initial 100k bloom loads in `qualityMode=ultra` with `rendererPixelRatio=0.62`; a sample after warmup showed `gpuRenderMs ~= 3.4`, `frameCpuMs ~= 4.5`, and no console errors.
- 2026-04-03: Verified `window.advanceTime(3000)` at `100000` petals. After advancing, the state moved to `onTree=98894 / falling=1106 / onGround=0`, confirming the Wasm sim and the high-count renderer stay in sync.
- 2026-04-03: Re-verified regression cases after reopening 100k. `setPetalCount(16000)` and `setPetalCount(24000)` still initialize correctly in `qualityMode=standard`, fullscreen still toggles on/off with `f`, and console errors remain 0.
- 2026-04-03: Visual inspection of the 100k path shows a deliberate fidelity tradeoff: once counts exceed `50000`, petals become denser, more solid, and slightly flatter in exchange for keeping the scene interactive.
- TODO: The 100k path is now implemented, but the main runtime cost at long mid-flight states shifts toward `wasm_step` + falling uploads once GPU overdraw is suppressed. If we want more headroom beyond 100k, the next step is reducing the falling upload path rather than touching tree activation again.
- 2026-04-03: Added environment asset integration from local `glTF/` using `GLTFLoader` and `MeshSurfaceSampler` in `index.html`. The scene now loads Quaternius grass, medium rocks, and stone-path models instead of relying only on procedural stand-ins.
- 2026-04-03: Shifted the backdrop / fog / island palette toward the blue studio look from the reference image so the sakura tree sits in a cleaner diorama space.
- 2026-04-03: Verified the asset pass with the web-game Playwright client against `http://127.0.0.1:3001/index.html`; fresh screenshots were written to `output/web-game/shot-0.png` and `output/web-game/shot-1.png`.
- 2026-04-03: Opened the latest screenshot and confirmed the new rocks, stepping stones, and scattered grass render in-scene. Also checked the page with Playwright MCP: console messages remained at 0 errors / 0 warnings.
- TODO: The current grass pass reads as scattered tufts rather than the dense velvet lawn in the target image. Next visual pass should either add another low-height ground-cover asset layer or swap to a denser grass model while keeping the same GLTF pipeline.
- 2026-04-03: Reworked the environment again to better match the reference diorama. Grass is now a dense procedural blade field via `MeshSurfaceSampler` + `InstancedMesh`, the island top is a hand-shaped organic cap instead of the old cylinder cap, and the side wall is a darker low-poly band.
- 2026-04-03: Replaced the textured asset rocks / path with lighter flat-shaded procedural stones so the scene reads more like a stylized mockup and less like dropped-in kit assets.
- 2026-04-03: Verified the revised ground pass in Playwright after the cleanup pass; the latest screenshots show much denser grass coverage and a slimmer dark pedestal closer to the target image.
