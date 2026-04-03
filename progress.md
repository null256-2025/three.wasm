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
- TODO: Move the current JS falling simulation state to Wasm SoA buffers from `sakura_final_spec_wasm.md`.
- TODO: Replace full falling-buffer uploads with partial uploads / active range updates in the falling layer.
