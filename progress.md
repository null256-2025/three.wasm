Original prompt: [sakura_final_spec.md](sakura_final_spec.md) を基に実装してください。画像のようなテイストがいいな

- 2026-04-02: `index.html` を最終仕様ベースで全面再構成。`InstancedMesh` 花びら2000枚、`ON_TREE -> FALLING -> ON_GROUND` の有限状態、リスポーンなし、島・木・岩・ルート・背景グラデーションを実装。
- 2026-04-02: 参照画像に寄せて、UIをミニマルな淡いピンク基調へ変更。背景はシェーダー球で柔らかいピンクグラデーションにした。
- 2026-04-02: `window.render_game_to_text` と `window.advanceTime(ms)` を追加。自動テストしやすい固定ステップ更新に変更。
- 2026-04-02: Playwright で確認。初期 1.22s 時点で `onTree=2000`、`advanceTime(3000)` 後は `onTree=1925 / falling=74 / onGround=1`、さらに `advanceTime(17000)` 後は `onTree=1342 / falling=147 / onGround=511` を確認。
- 2026-04-02: スクリーンショット確認で、初期の満開感が弱かったため花びらの向き・密度・構図・配色を再調整。
- NOTE: `f` キーによるフルスクリーン切替は実装済み。Playwright の通常操作では `document.fullscreenElement` を安定して確認しづらいため、必要なら実ブラウザで追加確認する。
