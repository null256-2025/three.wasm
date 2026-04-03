Original prompt: [sakura_final_spec.md](sakura_final_spec.md) を基に実装してください。画像のようなテイストがいいな

- 2026-04-02: `index.html` を最終仕様ベースで全面再構成。`InstancedMesh` 花びら 2000 枚、`ON_TREE -> FALLING -> ON_GROUND` の有限状態、リスポーンなし、島・木・岩・ルート・背景グラデーションを実装。
- 2026-04-02: 参照画像に寄せて、UI をミニマルな淡いピンク基調へ変更。背景はシェーダー球で柔らかいピンクグラデーションにした。
- 2026-04-02: `window.render_game_to_text` と `window.advanceTime(ms)` を追加。自動テストしやすい固定ステップ更新に変更。
- 2026-04-02: Playwright で確認。初期 1.22s 時点で `onTree=2000`、`advanceTime(3000)` 後は `onTree=1925 / falling=74 / onGround=1`、さらに `advanceTime(17000)` 後は `onTree=1342 / falling=147 / onGround=511` を確認。
- 2026-04-02: スクリーンショット確認で、初期の満開感が弱かったため花びらの向き・密度・構図・配色を再調整。
- 2026-04-02: 花びら数を固定値から可変に変更。右上にレンジ入力・数値入力・プリセットボタンを追加し、`window.setPetalCount(count)` と `?petals=24000` のような URL 指定でも反映できるようにした。
- 2026-04-02: 花びら数変更時は `InstancedMesh` とデータ配列を作り直し、満開状態から再初期化する設計にした。上限は現在 `100000`、500 枚刻み。
- 2026-04-02: Chrome DevTools で確認。`window.setPetalCount(24000)` 直後に `total=24000 / onTree=24000`、その後 `window.advanceTime(3000)` を呼んだ状態でも 24000 枚で遷移が継続することを確認。
- 2026-04-03: `petals` の UI 上限、クランプ上限、最大プリセット値を `100000` に引き上げた。
- NOTE: 現在の実装は WebAssembly を使っていない。花びら更新ループは JavaScript のままなので、枚数をさらに上げるならまず CPU コストの計測が必要。
- NOTE: `f` キーによるフルスクリーン切替は実装済み。自動ブラウザ操作では `document.fullscreenElement` を安定確認しづらいため、必要なら実ブラウザで追加確認する。
