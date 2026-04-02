# 🌸 桜の花びら物理演算デモ
## 要件定義書 & 実装仕様書
**three.js / three.wasm 対応版** — Version 1.0 | 2026年4月

---

## 1. プロジェクト概要

### 1-1. 目的

ローポリスタイルの桜の木から花びらが物理演算に従ってはらはらと落下するインタラクティブな3Dデモをブラウザ上に実装する。

花びら1枚1枚が独立したオブジェクトとして存在し、重力・風・回転などの物理挙動を持つ。将来的にthree.wasmへ移行することで花びら数を大幅に増やすことを見据えた設計とする。

### 1-2. 参照デザイン

以下のビジュアルを参照デザインとする（添付画像より）：

- ローポリ（low-poly）スタイルの3Dグラフィック
- 桜の木：濃いピンク（`#E8709A`〜`#F4A0C0`）の花房、濃い茶色（`#3D1C0F`）の幹・枝
- 地面：円形の島状、明るいグリーン（`#7EC850`）の草、黒に近いダークグレー（`#1A1A1A`）の断面
- 岩石：グレー（`#B0B0B0`〜`#D0D0D0`）のローポリ石
- 背景：明るいスカイブルー（`#87CEEB`）の単色
- 地面に散らばる花びら：ピンク系の小さなローポリ平面
- 全体的に影は柔らかく、メルヘン・かわいい雰囲気

### 1-3. 技術スタック

| 項目 | 採用技術 | 備考 |
|---|---|---|
| 3Dレンダリング | three.js r160+ | 将来的にthree.wasmへ移行想定 |
| 言語 | JavaScript（ES Module） | TypeScript不要、シンプルに |
| ビルドツール | なし（CDN直接読み込み） | index.html単体で動作 |
| 物理演算 | 自前実装（カスタム） | 外部物理ライブラリ不使用 |
| ホスティング | ローカルサーバー or GitHub Pages | `npx serve .` で起動 |

---

## 2. デザイン仕様

### 2-1. カラーパレット

| 要素 | カラーコード | 用途・補足 |
|---|---|---|
| 背景 | `#87CEEB` | スカイブルー単色（グラデーションなし） |
| 桜の花（明） | `#F4A0C0` | 花房外側・明るい部分 |
| 桜の花（中） | `#E8709A` | 花房中央・基本色 |
| 桜の花（暗） | `#C8507A` | 花房内側・影になる部分 |
| 幹・枝 | `#3D1C0F` | 濃い茶色 |
| 草（明） | `#8ED460` | 地面上面・明るい側 |
| 草（暗） | `#6AAA40` | 地面上面・影になる側 |
| 地面断面 | `#1A1A1A` | 島の側面・ほぼ黒 |
| 岩（明） | `#D0D0D0` | ライトグレー |
| 岩（暗） | `#909090` | ダークグレー |
| 落下花びら | `#F0B0C8` | 地面に落ちた花びら |

### 2-2. ジオメトリ仕様

#### ■ 桜の木

- **幹**：`CylinderGeometry`（底面半径 0.15、上面半径 0.08、高さ 2.5）
- **主枝**：3〜5本、幹の上部からランダム方向に伸びる `CylinderGeometry`
- **小枝**：主枝からさらに分岐、2〜3本
- **花房**：球体クラスター。`SphereGeometry`（半径 0.4〜0.7）を 20〜30個、木の上部にランダム配置
- **花房ポリゴン数**：各球体を低めに抑える（`widthSegments: 5, heightSegments: 4`）でローポリ感を出す

> 💡 花房同士は重なってもOK。密度が高いほどリッチに見える

#### ■ 地面（島）

- **上面**：`CylinderGeometry`（半径 3.5、高さ 0.3、`radialSegments: 8`）でローポリ八角形に
- **断面**：同じ CylinderGeometry の側面を別マテリアルで黒塗り
- **草のランダム凹凸**：頂点をわずかにY方向にランダムオフセット（±0.05）してローポリ感を強調

#### ■ 岩

- 3〜5個配置。`BoxGeometry` または `IcosahedronGeometry`（`detail: 0`）を変形
- 各岩は `scale.set` でランダムにつぶす（例：`scale(1.0, 0.6, 0.8)`）
- 地面中央〜やや右寄りに自然な感じで配置
- 岩の頂点をわずかにランダムオフセットしてより自然な形に

#### ■ 花びら（落下する物体）

- **形状**：`PlaneGeometry`（幅 0.12、高さ 0.10）
- **ポリゴン数**：1枚あたり2ポリゴン（最小限）
- **初期数**：500枚（パフォーマンス調整用のパラメータとして定数化）
- **最大数**：5,000枚（three.wasm移行後の目標値）
- **両面表示**：`MeshBasicMaterial` の `side: THREE.DoubleSide` を設定

> ⚠️ `InstancedMesh` を使うこと。個別の Mesh を500個生成すると DrawCall が爆発する

### 2-3. ライティング

| ライト種類 | 設定値 | 役割 |
|---|---|---|
| `AmbientLight` | color: `#FFF8F0`, intensity: 0.8 | 全体を柔らかく照らす（影なし） |
| `DirectionalLight` | color: `#FFFFFF`, intensity: 1.2, pos: (5, 10, 5) | 主光源・太陽光 |
| `DirectionalLight`（補助） | color: `#C8E8FF`, intensity: 0.4, pos: (-3, 5, -3) | 反射光・影を和らげる |

> 💡 影（Shadow）は重いため最初は無効。必要なら `DirectionalLight` の `castShadow` を有効化してオプション設定

### 2-4. カメラ・構図

- **カメラ**：`PerspectiveCamera`（fov: 45, near: 0.1, far: 100）
- **初期位置**：`position(0, 4, 8)`、`lookAt(0, 1.5, 0)`
- **コントロール**：`OrbitControls`（ドラッグで回転、ホイールでズーム）
- **自動回転**：`autoRotate: true`、`autoRotateSpeed: 0.3`（ゆっくり回す）
- **ズーム範囲**：`minDistance: 3`、`maxDistance: 15`

> 💡 自動回転でパッシブに見ていても絵になる構図を保つ

---

## 3. 物理演算仕様

### 3-1. 花びら1枚の状態変数

| 変数名 | 型 | 初期値 | 説明 |
|---|---|---|---|
| `position` | Vector3 | 木の花房付近のランダム位置 | 現在位置 |
| `velocity` | Vector3 | (±0.02, 0, ±0.02) ランダム | 現在速度 |
| `rotation` | Euler | ランダム | 現在の向き |
| `rotationSpeed` | Vector3 | 各軸ランダム (±0.03) | 回転速度 |
| `windPhase` | float | ランダム(0〜2π) | 風のゆらぎ用位相オフセット |
| `fallSpeed` | float | 0.008〜0.015 ランダム | 基本落下速度 |
| `active` | bool | true | 地面着地後は false |
| `groundY` | float | 0.01 | 着地判定のY座標 |

### 3-2. 毎フレームの更新ロジック（疑似コード）

各花びらに対して以下の計算を毎フレーム実行する：

```javascript
// 1. 重力
velocity.y -= 0.0008;

// 2. 風（時間と位相によるサイン波）
const wind = Math.sin(time * 0.8 + petal.windPhase) * 0.003;
velocity.x += wind;
velocity.z += Math.cos(time * 0.6 + petal.windPhase) * 0.002;

// 3. 速度に減衰（空気抵抗）
velocity.multiplyScalar(0.98);

// 4. 位置を更新
position.add(velocity);

// 5. 回転を更新
rotation.x += rotationSpeed.x;
rotation.z += rotationSpeed.z;

// 6. 地面判定
if (position.y < groundY) {
  position.y = groundY;
  velocity.set(0, 0, 0);      // 止まる
  rotationSpeed.set(0, 0, 0);
  petal.active = false;
}

// 7. InstancedMesh の Matrix を更新
dummy.position.copy(position);
dummy.rotation.copy(rotation);
dummy.updateMatrix();
instancedMesh.setMatrixAt(i, dummy.matrix);
```

### 3-3. 花びらの生成サイクル

- 着地して `active: false` になった花びらは、一定時間後に木の上部でリセット（リサイクル）
- リセット位置：木の花房クラスター内のランダムな点（半径1.5以内）
- 常に一定数が落下中になるよう制御
- 初回は全花びらを時間をずらして（stagger）生成し、最初から全部が同時に落ち始めないようにする

> 💡 花びらを使い捨てにすると数が増えたときにメモリを圧迫する。必ずリサイクル設計にすること

### 3-4. パフォーマンス目標

| 花びら枚数 | 目標fps | 実装方式 |
|---|---|---|
| 500枚 | 60fps以上 | InstancedMesh（デフォルト） |
| 2,000枚 | 60fps以上 | InstancedMesh |
| 5,000枚 | 60fps以上 | InstancedMesh + three.wasm移行後 |
| 10,000枚〜 | 検証中 | three.wasm専用 |

---

## 4. ファイル構成

### 4-1. ディレクトリ構成

```
sakura-demo/
├── index.html          # エントリーポイント（全コードをここに集約）
└── README.md           # 起動手順
```

> 💡 `index.html` 1ファイルで完結させること。外部画像・フォント等は使わない

### 4-2. index.html の CDN 設定

```html
<script type="importmap">
{
  "imports": {
    "three": "https://cdn.jsdelivr.net/npm/three@0.160/build/three.module.js",
    "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.160/examples/jsm/"
  }
}
</script>

<script type="module">
  import * as THREE from 'three';
  import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
  // ↑ これだけでOK。他の外部依存なし
</script>
```

### 4-3. コードの内部モジュール分割（同一ファイル内で関数に分ける）

| 関数名 | 役割 |
|---|---|
| `initScene()` | Scene / Camera / Renderer / Controls の初期化 |
| `createGround()` | 地面の島ジオメトリ生成 |
| `createTree()` | 幹・枝・花房の生成 |
| `createRocks()` | 岩の生成・配置 |
| `createPetals(count)` | InstancedMesh と花びらデータ配列の初期化 |
| `updatePetals(time)` | 毎フレームの物理演算・Matrix更新 |
| `onWindowResize()` | ウィンドウリサイズ対応 |
| `animate()` | requestAnimationFrame のループ |

---

## 5. 実装手順（AIエージェント向け）

### 5-1. 実装の優先順位

以下の順番で実装すること。各ステップで動作確認してから次に進むこと：

| ステップ | 内容 | 完了の確認方法 |
|---|---|---|
| Step 1 | Scene・背景色・カメラ・OrbitControls のセットアップ | スカイブルーの画面が表示される |
| Step 2 | 地面（島）の生成 | 緑の円形の島が見える |
| Step 3 | 幹・枝の生成 | 茶色の木の骨格が立っている |
| Step 4 | 花房（ピンクの球体群）の追加 | 木がピンクの花で覆われて見える |
| Step 5 | 岩の配置 | 地面に灰色の岩が置かれている |
| Step 6 | InstancedMesh で花びら生成（静止状態） | 木の周辺にピンクの板が浮かんでいる |
| Step 7 | 物理演算ループの実装 | 花びらがはらはらと落下する |
| Step 8 | 着地・リサイクル処理 | 連続して花びらが降り続ける |
| Step 9 | 自動回転・仕上げ | OrbitControls の自動回転で映える |

### 5-2. 実装時の禁止事項

> ⚠️ 個別の `Mesh`（`new THREE.Mesh`）を花びらの枚数分だけ生成しないこと → 必ず `InstancedMesh` を使う

> ⚠️ 花びらに毎フレーム `new Vector3()` などで新規オブジェクトを生成しないこと → GC の原因になる

> ⚠️ 外部の物理ライブラリ（cannon.js、ammo.js など）を使わないこと → 自前実装で十分

> ⚠️ `async/await` で外部リソースを読み込まないこと → 完全オフライン動作を維持

> ⚠️ TypeScript やバンドラー（Vite 等）を使わないこと → index.html 単体完結

### 5-3. パラメータ定数（ファイル先頭に定義）

以下の定数をファイル先頭にまとめて定義し、後から調整しやすくすること：

```javascript
const PETAL_COUNT       = 500;    // 花びら枚数
const PETAL_SIZE        = 0.12;   // 花びらサイズ
const GRAVITY           = 0.0008; // 重力加速度
const WIND_STRENGTH     = 0.003;  // 風の強さ
const AIR_RESISTANCE    = 0.98;   // 空気抵抗（1に近いほど軽い）
const FALL_SPEED_MIN    = 0.008;  // 落下速度（最小）
const FALL_SPEED_MAX    = 0.015;  // 落下速度（最大）
const GROUND_Y          = 0.01;   // 着地判定のY座標
const AUTO_ROTATE_SPEED = 0.3;    // 自動回転速度
```

### 5-4. 起動方法

```bash
# 1. フォルダに移動
cd sakura-demo

# 2. ローカルサーバーを起動
npx serve .

# 3. ブラウザで開く
# http://localhost:3000
```

> ⚠️ `file://` での直接開封は CORS エラーになる。必ずローカルサーバーを使うこと

---

## 6. 将来的な three.wasm 移行について

### 6-1. 移行時に変わること

| 項目 | three.js版（現在） | three.wasm版（将来） |
|---|---|---|
| 花びら枚数の限界 | 〜2,000枚程度 | 50,000枚以上を目標 |
| 物理演算の場所 | JavaScript（メインスレッド） | Wasmバイナリ内 |
| GCによるカクつき | 発生しうる | ほぼなし |
| API | three.js 標準API | Wasm用の独自API |
| ファイルサイズ | 〜600KB（three.js本体） | 〜10KB（Wasmバイナリ） |

### 6-2. 移行しやすい設計のポイント

- 物理演算ロジックを `updatePetals()` 関数に完全に閉じ込めておく
- 花びらの状態を配列（`petalData[]`）で一元管理し、Wasmへの受け渡しを容易にする
- InstancedMesh の Matrix 更新部分を `updatePetals()` の最後にまとめて行う

> 💡 three.wasm への移行時は `updatePetals()` の中身をWasm呼び出しに置き換えるだけで済む設計にする

---

## 7. 完成イメージ確認チェックリスト

実装完了後、以下をすべて確認すること：

| 確認項目 | 期待値 |
|---|---|
| 背景色 | スカイブルー（`#87CEEB`）の単色 |
| 桜の木 | 茶色の幹・枝にピンクの花房が自然に見える |
| 地面 | ローポリ八角形の緑の島、側面は黒 |
| 岩 | 3個以上のグレーのローポリ岩 |
| 花びら | はらはらとランダムに落下し続けている |
| 落下の動き | 風に揺れながら回転しつつ落ちる |
| 着地後 | 地面に静止し、その後リサイクルされる |
| fps | 500枚で60fps以上をキープ |
| ドラッグ操作 | 自由に視点を回転できる |
| 自動回転 | ゆっくりとシーンが自動回転する |
| レスポンシブ | ウィンドウリサイズで正常に対応 |

---

*以上*
