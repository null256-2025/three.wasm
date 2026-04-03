# 🌸 桜の花びら物理演算デモ — AIエージェント向け最終実装指示書

**Version FINAL | ルートA: Rust + Wasm | 2026年4月**

> **この指示書について**
> 本ドキュメントは以下の4つの設計書を統合・精査した唯一の正です。
>
> - 初期仕様書（v1）— ビジュアル仕様・カラーパレット・ジオメトリ定義
> - 修正指示書（v2）— GLSLシェーダー・地面修正
> - コンセプト修正（v3）— 花びらの一生（有限・永久残留・リサイクル禁止）
> - **Wasm Redesign 設計書** — 責務分離・SoA・3レイヤー・ブリッジAPI
>
> 過去の仕様書は参照不要。**本書だけに従うこと。**

---

## 0. 最重要事項

### 0-1. 花びらのコンセプト

```
【花びらの一生 — リサイクルしない】

① ON_TREE   ：木の枝に咲いている      ← 初期状態。全枚数が木にある＝満開
      ↓ fallDelay フレーム経過後、1枚ずつ離脱
② FALLING   ：空中を舞いながら落下する  ← 物理演算の対象（Wasm が担当）
      ↓ 地面に到達
③ ON_GROUND ：地面に静止する           ← 永久残留。消えない。触らない
```

- 花びら総数は **PETAL_COUNT（デフォルト 2,000、Wasm で 100,000 を目標）**
- 起動直後＝満開 → 徐々に散る → 全部散ると木が裸、地面が花びらで覆われる
- **リサイクル・リスポーンは一切しない。`resetPetal()` のような関数は存在しない**

### 0-2. なぜ Wasm を使うのか — 現行設計の4つの問題

| # | 問題 | 説明 |
|---|---|---|
| 1 | 全件ループ | `updatePetals()` が全花びらを毎フレーム走査。ON_TREE も ON_GROUND も巻き込まれる |
| 2 | 行列更新中心 | 各花びらに `dummy.updateMatrix()` + `setMatrixAt()` → 数が増えると CPU で詰まる |
| 3 | 状態ごとの差が薄い | ON_TREE・ON_GROUND は本来「静的」なのに毎フレーム処理パスに入る |
| 4 | Wasm を入れる場所がない | AoS（オブジェクトの配列）構造では Wasm に渡しても恩恵がない |

### 0-3. 目標構成

```
ON_TREE   → GPU（頂点シェーダー）が揺れを担当。CPU 更新なし
FALLING   → Wasm（Rust）が物理計算。active 分だけ更新
ON_GROUND → 静的。着地時に1回だけ書き込み。以後触らない
JS        → orchestration（指揮）のみ
```

> ⚠️ `mrdoob/three.wasm` は 2026 年 4 月 1 日公開のエイプリルフールジョーク（キューブを1個回すだけの10KBバイナリ）。
> 本プロジェクトでは使用しない。
> 採用するのは **「Three.js（レンダリング）+ 自前 Rust → Wasm モジュール（シミュレーション）」の
> ハイブリッド構成** である。

### 0-4. 木モデル方針

- 木は **見た目優先** とし、`proctree.js` による自然な枝分かれを維持する
- 軽量化の主戦場は木メッシュではなく、**花びらの状態更新と GPU 転送** である
- `BLOSSOM_POSITIONS` は `tree.twigVerts` から収集し、満開感は ON_TREE 花びらで作る
- Wasm は木メッシュを描くためではなく、**花びらシミュレーションの高速化** に集中させる
- Phase 1 の公開 UI 上限は安全側に絞る。10 万枚目標は Phase 2/3 完了後に再開する

---

## 1. 技術スタック

| 項目 | 採用技術 | 備考 |
|---|---|---|
| 3D レンダリング | three.js r160+ | CDN 読み込み |
| 木の生成 | proctree.js | L-system プロシージャル木生成 |
| 花びら物理計算 | **Rust → WebAssembly** | `wasm-bindgen` + `wasm-pack` |
| 描画方式 | `InstancedBufferGeometry` × 3 レイヤー | 状態別に分離 |
| 花びら形状 | GLSL `ShaderMaterial`（フラグメントシェーダー） | 花びら型カットアウト |
| ON_TREE 揺れ | GLSL 頂点シェーダー | `sin(time + phase)` で GPU 計算 |
| ビルド | `wasm-pack build --target web --release` | Wasm バイナリ生成 |

---

## 2. ファイル構成

```
sakura-demo/
├── index.html                # エントリーポイント（JS + シェーダー + Wasm ブリッジ）
├── Cargo.toml                # Rust プロジェクト設定
├── src/
│   └── lib.rs                # Rust ソース（花びらシミュレーション）
├── pkg/                      # wasm-pack が生成
│   ├── sakura_sim_bg.wasm    # Wasm バイナリ
│   └── sakura_sim.js         # JS グルーコード
└── README.md
```

---

## 3. 責務分離（この設計の核心）

### 3-1. JavaScript の責務

- three.js scene 初期化（カメラ・ライト・コントロール）
- proctree.js による木の生成、`BLOSSOM_POSITIONS` 収集
- 地面・岩などの静的オブジェクト生成
- Wasm メモリの確保とブリッジ（`Float32Array` ビューの貼り付け）
- 3つの描画レイヤーの管理と GPU 転送
- フレームループの指揮（orchestration）
- デバッグ UI

### 3-2. Wasm（Rust）の責務

- **FALLING 花びらの位置・速度・回転の更新**（重力・風・空気抵抗）
- `fallDelay` 到達判定（ON_TREE → FALLING 遷移）
- 着地判定（FALLING → ON_GROUND 遷移）
- `activeIndices` リストの管理と compaction
- 着地イベント `landedIndices` の生成と JS への返却

### 3-3. GPU（シェーダー）の責務

- ON_TREE 花びらの微揺れ（頂点シェーダーで `time + phase` → CPU 更新不要）
- 花びら形状のカットアウト（フラグメントシェーダー）
- ON_TREE / ON_GROUND / FALLING の最終頂点変換

---

## 4. データレイアウト — SoA（Structure of Arrays）

### ❌ 禁止: AoS（オブジェクトの配列）

```javascript
// これはやらない
petalData.push({
  position: new THREE.Vector3(x, y, z),
  velocity: new THREE.Vector3(vx, vy, vz),
  // ...オブジェクト参照が多く、Wasm や高速ループに不向き
});
```

### ✅ 採用: SoA（Wasm 共有メモリ上に確保）

```
// 全花びら分（PETAL_COUNT 個）
posX[N]         posY[N]         posZ[N]          // Float32Array — 位置
velX[N]         velY[N]         velZ[N]          // Float32Array — 速度
rotX[N]         rotY[N]         rotZ[N]          // Float32Array — 回転角
rotSpeedX[N]    rotSpeedY[N]    rotSpeedZ[N]     // Float32Array — 回転速度
phase[N]                                          // Float32Array — 風のゆらぎ位相
scale[N]                                          // Float32Array
fallDelay[N]                                      // Int32Array   — 離脱までの待機フレーム
state[N]                                          // Uint8Array   — 0=ON_TREE, 1=FALLING, 2=ON_GROUND

// FALLING 専用
activeIndices[activeCount]                        // Uint32Array — FALLING 状態の花びら index

// 着地イベント（毎フレームクリアされる）
landedIndices[landedCount]                        // Uint32Array — このフレームで着地した花びら index
```

すべて Wasm の `memory.buffer` 上に確保し、JS 側から `new Float32Array(memory.buffer, ptr, count)` で **ゼロコピー参照** する。

---

## 5. 描画レイヤー構成（3つの InstancedMesh に分離）

### Layer A: Tree Petals（ON_TREE）

| 項目 | 値 |
|---|---|
| 対象 | ON_TREE 状態の花びら |
| 実装 | `InstancedBufferGeometry` + カスタム `ShaderMaterial` |
| attribute | `basePosition`, `baseRotation`, `phase`, `scale` |
| uniform | `time`（毎フレーム更新するのはこれだけ） |
| 揺れ | **頂点シェーダー**で `sin(time * 0.5 + phase) * 0.01` 等 |
| 離脱時 | `scale = 0` にして非表示化、または最後尾の instance と swap |
| CPU 更新 | **なし** |

### Layer B: Falling Petals（FALLING）

| 項目 | 値 |
|---|---|
| 対象 | FALLING 状態の花びら |
| 実装 | `InstancedBufferGeometry` + カスタム `ShaderMaterial` |
| attribute | `aPosition`, `aRotation`, `aScale`, `aBrightness` |
| 更新 | **Wasm 計算結果を `activeCount` 分だけ partial upload** |
| 全件ループ | **禁止。active 範囲のみ** |

### Layer C: Ground Petals（ON_GROUND）

| 項目 | 値 |
|---|---|
| 対象 | ON_GROUND 状態の花びら |
| 実装 | `InstancedBufferGeometry` + カスタム `ShaderMaterial` |
| 書き込み | 着地時に 1 回だけスロットに書き込む |
| CPU 更新 | **なし** |

---

## 6. Wasm ブリッジ API

### 6-1. JS → Wasm 呼び出し

```
initPetalBuffers(count: u32)
  → SoA バッファを Wasm メモリ上に確保

seedPetals(seed: u32, blossomPositionsPtr: *f32, blossomCount: u32)
  → 花房位置から全花びらの初期データ（treePosition, fallDelay, phase 等）を生成

activateReadyPetals(frameCount: u32) → u32
  → fallDelay 到達した花びらを ON_TREE → FALLING に遷移
  → 戻り値: 新たに FALLING になった花びら数

stepFalling(dt: f32, time: f32, windStrength: f32, gravity: f32, airResistance: f32,
            islandRadius: f32, groundY: f32, groundYOuter: f32)
  → FALLING の花びらだけを SoA 上で物理更新
  → 着地した花びらを landedIndices に積む

getActiveCount() → u32          // 現在 FALLING 中の数
getLandedCount() → u32          // このフレームで着地した数

// ポインタ取得（JS が typed array view を貼るため）
getPosXPtr() → *f32
getPosYPtr() → *f32
getPosZPtr() → *f32
getRotXPtr() → *f32
getRotYPtr() → *f32
getRotZPtr() → *f32
getStatePtr() → *u8
getActiveIndicesPtr() → *u32
getLandedIndicesPtr() → *u32
getPhasePtr() → *f32
getScalePtr() → *f32
getTreePosXPtr() → *f32
getTreePosYPtr() → *f32
getTreePosZPtr() → *f32
```

### 6-2. JS 側のメモリ参照

```javascript
const mem = wasm.memory.buffer;
const posX = new Float32Array(mem, wasm.getPosXPtr(), PETAL_COUNT);
const posY = new Float32Array(mem, wasm.getPosYPtr(), PETAL_COUNT);
const posZ = new Float32Array(mem, wasm.getPosZPtr(), PETAL_COUNT);
// ...同様に全 SoA 配列をゼロコピー参照
```

> ⚠️ Wasm memory が `grow` するとすべての `ArrayBuffer` が無効化される。
> `grow` 後は必ず全ビューを再取得すること。

---

## 7. フレームループ（新設計）

```
1. JS: now, dt, time を計算
2. JS → Wasm: activateReadyPetals(frameCount)
     → fallDelay 到達した花びらを ON_TREE → FALLING に遷移
     → 戻り値で新規 FALLING 数を取得
3. JS: 新規 FALLING があれば Layer A (Tree) の該当 instance を非表示化（scale=0）
4. JS → Wasm: stepFalling(dt, time, windStrength, gravity, airResistance, ...)
     → FALLING の花びらだけを SoA 上で物理更新
     → 着地した花びらを landedIndices に積む
5. JS: getLandedCount() + getLandedIndicesPtr() で着地イベントを取得
     → Layer C (Ground) に着地分だけ 1 回書き込む
6. JS: Layer B (Falling) の attribute を activeCount 分だけ GPU に partial upload
     → bufferAttribute.updateRange を使い、active 範囲だけ転送
7. JS: Layer A (Tree) の uniform time を更新（揺れ用）
8. JS: controls.update()
9. JS: renderer.render(scene, camera)
```

**ポイント：**
- ON_TREE 全件更新なし（uniform time だけ）
- ON_GROUND 全件更新なし（着地時に1回書くだけ）
- FALLING の active 範囲のみ更新（全件ループ禁止）
- JS は orchestration のみ

---

## 8. Rust 実装仕様

### 8-1. Cargo.toml

```toml
[package]
name = "sakura-sim"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
opt-level = 3
lto = true
```

### 8-2. 状態定数

```rust
const ON_TREE: u8 = 0;
const FALLING: u8 = 1;
const ON_GROUND: u8 = 2;
```

### 8-3. `step_falling()` の疑似コード

```rust
#[wasm_bindgen]
pub fn step_falling(
    dt: f32, time: f32,
    wind_strength: f32, gravity: f32, air_resistance: f32,
    island_radius: f32, ground_y: f32, ground_y_outer: f32,
) {
    landed_count = 0;
    let mut write_idx = 0;

    for read_idx in 0..active_count {
        let idx = active_indices[read_idx] as usize;

        // 重力
        vel_y[idx] -= gravity;

        // 風（複数周波数を重ねて自然に）
        let ws = wind_strength;
        let p = phase[idx];
        let wind = (time * 0.8 + p).sin() * ws
                 + (time * 1.7 + p * 1.3).sin() * ws * 0.4
                 + (time * 0.3 + p * 0.7).sin() * ws * 0.6;
        vel_x[idx] += wind;
        vel_z[idx] += (time * 0.6 + p).cos() * ws * 0.8;

        // 空気抵抗
        vel_x[idx] *= air_resistance;
        vel_y[idx] *= air_resistance;
        vel_z[idx] *= air_resistance;

        // 位置更新
        pos_x[idx] += vel_x[idx];
        pos_y[idx] += vel_y[idx];
        pos_z[idx] += vel_z[idx];

        // 回転更新
        rot_x[idx] += rot_speed_x[idx];
        rot_z[idx] += rot_speed_z[idx];

        // 着地判定
        let dist = (pos_x[idx] * pos_x[idx] + pos_z[idx] * pos_z[idx]).sqrt();
        let gl = if dist < island_radius { ground_y } else { ground_y_outer };

        if pos_y[idx] <= gl {
            // 着地処理
            pos_y[idx] = gl;
            vel_x[idx] = 0.0; vel_y[idx] = 0.0; vel_z[idx] = 0.0;
            rot_x[idx] = -PI / 2.0 + small_random;  // 水平に寝かせる
            rot_z[idx] = small_random;
            state[idx] = ON_GROUND;
            landed_indices[landed_count] = idx as u32;
            landed_count += 1;
            // active_indices から除去（write_idx を進めない）
        } else {
            // まだ FALLING → compaction（詰める）
            active_indices[write_idx] = idx as u32;
            write_idx += 1;
        }
    }

    active_count = write_idx as u32;
}
```

> 💡 compaction は `step_falling` 内で同時に行う（別パスにしない）。
> ループ内で `read_idx` は全件走査、`write_idx` は着地済みを飛ばして詰めていく。

### 8-4. `activate_ready_petals()` の疑似コード

```rust
#[wasm_bindgen]
pub fn activate_ready_petals(frame_count: u32) -> u32 {
    let mut newly_activated = 0;

    for i in 0..petal_count {
        if state[i] == ON_TREE && fall_delay[i] <= frame_count as i32 {
            state[i] = FALLING;
            // treePosition → position にコピー（落下開始位置）
            pos_x[i] = tree_pos_x[i];
            pos_y[i] = tree_pos_y[i];
            pos_z[i] = tree_pos_z[i];
            // 初速を設定
            vel_x[i] = small_random_x;
            vel_y[i] = -small_random_y;
            vel_z[i] = small_random_z;
            // active list に追加
            active_indices[active_count] = i as u32;
            active_count += 1;
            newly_activated += 1;
        }
    }

    newly_activated
}
```

> ⚠️ `activate_ready_petals` は全件走査が発生するが、毎フレーム数枚しか遷移しないため
> ON_TREE の数が減るにつれて走査対象も減っていく。
> Phase 3 以降で「ON_TREE を別リストで管理」する最適化が可能。

### 8-5. 乱数について

Wasm 内で乱数が必要な箇所（初速・着地時の微調整）には、
シンプルな xorshift32 を Rust 内に実装する。外部クレート不要。

```rust
static mut RNG_STATE: u32 = 12345;

fn xorshift32() -> u32 {
    unsafe {
        RNG_STATE ^= RNG_STATE << 13;
        RNG_STATE ^= RNG_STATE >> 17;
        RNG_STATE ^= RNG_STATE << 5;
        RNG_STATE
    }
}

fn rand_f32() -> f32 {
    (xorshift32() as f32) / (u32::MAX as f32)
}
```

### 8-6. ビルド

```bash
wasm-pack build --target web --release
# → pkg/ に sakura_sim_bg.wasm と sakura_sim.js が生成される
```

---

## 9. シェーダー仕様

### 9-1. ON_TREE 頂点シェーダー（GPU 揺れ）

```glsl
attribute vec3 basePosition;
attribute vec3 baseRotation;
attribute float aPhase;
attribute float aScale;
uniform float time;
varying vec2 vUv;
varying float vBrightness;

// 簡易回転行列（Y軸回転のみで十分）
mat3 rotateY(float angle) {
  float s = sin(angle); float c = cos(angle);
  return mat3(c, 0, s, 0, 1, 0, -s, 0, c);
}

void main() {
  vUv = uv;
  vBrightness = 0.9 + fract(sin(float(gl_InstanceID) * 12.9898) * 43758.5453) * 0.2;

  // GPU で揺れを計算（CPU 更新不要）
  vec3 pos = basePosition;
  pos.x += sin(time * 0.5 + aPhase) * 0.01;
  pos.y += sin(time * 0.3 + aPhase * 1.3) * 0.005;
  pos.z += cos(time * 0.4 + aPhase) * 0.01;

  // 回転適用
  vec3 transformed = rotateY(baseRotation.y) * position * aScale;
  transformed += pos;

  gl_Position = projectionMatrix * modelViewMatrix * vec4(transformed, 1.0);
}
```

### 9-2. FALLING / ON_GROUND 頂点シェーダー

```glsl
attribute vec3 aPosition;
attribute vec3 aRotation;
attribute float aScale;
varying vec2 vUv;
varying float vBrightness;

void main() {
  vUv = uv;
  vBrightness = 0.9 + fract(sin(float(gl_InstanceID) * 12.9898) * 43758.5453) * 0.2;

  // Wasm（FALLING）または JS（ON_GROUND、着地時1回）が書き込んだ値をそのまま使う
  float cx = cos(aRotation.x), sx = sin(aRotation.x);
  float cz = cos(aRotation.z), sz = sin(aRotation.z);
  mat3 rotMat = mat3(cz, -sz, 0, sx*sz, sx*cz, cx, 0, 0, 1); // 簡略化

  vec3 transformed = rotMat * position * aScale;
  transformed += aPosition;

  gl_Position = projectionMatrix * modelViewMatrix * vec4(transformed, 1.0);
}
```

### 9-3. フラグメントシェーダー（全レイヤー共通）

```glsl
uniform vec3 color;
varying vec2 vUv;
varying float vBrightness;

void main() {
  vec2 uv = vUv - 0.5;

  // 花びら形状：3つの円の union
  float d1 = length(uv - vec2(-0.18, 0.05)) - 0.28;
  float d2 = length(uv - vec2( 0.18, 0.05)) - 0.28;
  float d3 = length(uv - vec2( 0.0, -0.15)) - 0.22;
  float d = min(min(d1, d2), d3);

  float alpha = 1.0 - smoothstep(-0.02, 0.02, d);
  if (alpha < 0.1) discard;

  vec3 c = color * vBrightness;
  c += (1.0 - length(uv) * 2.0) * 0.08; // 中心ほど白
  gl_FragColor = vec4(c, alpha);
}
```

---

## 10. 静的オブジェクト仕様

### 10-1. 地面（島）— `createGround()`

**① 島の上面（草）：** `CylinderGeometry(3.2, 3.2, 0.02, 10)`, `color: 0x7EC850`, `y = 0.31`

**② 島の本体：** `CylinderGeometry(3.2, 3.8, 0.6, 10)` — 上が狭く下が広い
- 頂点ランダムオフセット X,Z ±0.15
- マルチマテリアル `[soilMat(0x5C3A1E), grassMat(0x7EC850), soilMat]`
- グループ順序: `0=側面, 1=上面cap, 2=下面cap`
- `y = 0.01`

**③ 背景草原：** `PlaneGeometry(120, 120, 12, 12)`, `color: 0x5A9E35`, `y = -0.20`, 頂点Z ±0.4

### 10-2. 木 — `createTree()`

**proctree.js パラメータ：**
```javascript
{
  clumpMax: 0.454, clumpMin: 0.404,
  lengthFalloffFactor: 0.85, lengthFalloffPower: 0.99,
  branchFactor: 2.8, radiusFalloffRate: 0.6,
  climbRate: 0.5, trunkKink: 0.09,
  maxRadius: 0.12, treeSteps: 5,
  taperRate: 0.95, twistRate: 3.02,
  segments: 6, levels: 4, sweepAmount: 0,
  initialBranchLength: 0.85, trunkLength: 1.8,
  dropAmount: -0.1, growAmount: 0.235,
  vMultiplier: 0.36, twigScale: 0.35, seed: 262
}
```

- `tree.verts` / `tree.faces` → `BufferGeometry` → `MeshLambertMaterial({ color: 0x3D1C0F })`
- `trunkMesh.position.y = 0.3`
- `tree.twigVerts` を 3 つおきに `BLOSSOM_POSITIONS` に収集（Y +0.3）
- 密度補完: 各位置の周囲にランダム 3 点追加
- **花房オブジェクト（ピンク球体）は生成しない** — ON_TREE 花びらが満開を表現

### 10-3. 岩 — `createRocks()`

4 個の `IcosahedronGeometry(0.5, 1)` を変形。設定値:
```javascript
[
  { x: 0.8, z: 0.5, sx: 0.35, sy: 0.28, sz: 0.32, ry: 0.3 },
  { x:-0.5, z: 0.8, sx: 0.28, sy: 0.22, sz: 0.30, ry: 1.1 },
  { x: 0.3, z:-0.7, sx: 0.42, sy: 0.25, sz: 0.35, ry: 2.2 },
  { x:-0.9, z:-0.3, sx: 0.20, sy: 0.18, sz: 0.22, ry: 0.8 },
]
```
頂点ランダムオフセット X,Z ±0.15, Y ±0.10。色は明度 0.65〜0.85 のグレー。

---

## 11. カラーパレット・ライティング・カメラ

### カラーパレット

| 要素 | コード |
|---|---|
| 背景 | `#87CEEB` |
| フォグ | `#B0D8F0`（near: 30, far: 80） |
| 花びら | `#F0B0C8` |
| 幹・枝 | `#3D1C0F` |
| 草（島上面） | `#7EC850` |
| 土（島側面） | `#5C3A1E` |
| 背景草原 | `#5A9E35` |

### ライティング

| ライト | color | intensity | position |
|---|---|---|---|
| AmbientLight | `#FFF8F0` | 0.8 | — |
| DirectionalLight | `#FFFFFF` | 1.2 | (5, 10, 5) |
| DirectionalLight(補助) | `#C8E8FF` | 0.4 | (-3, 5, -3) |

影は無効。

### カメラ

- `PerspectiveCamera(45, aspect, 0.1, 100)`, 初期位置 `(0, 4, 8)` → `lookAt(0, 1.5, 0)`
- `OrbitControls`: autoRotate 0.3, enableDamping 0.05, min 3, max 15
- `controls.target.set(0, 1.5, 0)`

---

## 12. パラメータ定数

```javascript
const PETAL_COUNT    = 2000;    // Phase 2 以降で 100,000 を目標
const PETAL_SIZE     = 0.10;
const GROUND_Y       = 0.32;
const GROUND_Y_OUTER = -0.20;
const ISLAND_RADIUS  = 3.2;

const PARAMS = {
  windStrength:  0.002,
  gravity:       0.0006,
  airResistance: 0.97,
};
```

---

## 13. 段階的実装順序

### Phase 1: JS のみで 3 レイヤー分離（Wasm なし）

| Step | 内容 | 確認方法 |
|---|---|---|
| 1 | `initScene()` — Scene / Camera / Renderer / Controls / Lights | スカイブルー画面 |
| 2 | `createGround()` — 島 + 草原 | 緑の島、側面が土色 |
| 3 | `createTree()` — proctree.js + BLOSSOM_POSITIONS 収集 | 枝分かれした茶色の木 |
| 4 | `createRocks()` | 灰色のローポリ岩 4 個 |
| 5 | Layer A (Tree) — ON_TREE 花びらを `InstancedBufferGeometry` + 頂点シェーダー揺れ | 満開に見える |
| 6 | Layer B (Falling) — FALLING を JS で CPU 更新（暫定。Phase 2 で Wasm に置換） | 花びらが散る |
| 7 | Layer C (Ground) — 着地時に 1 回だけ書き込む静的レイヤー | 地面に花びらが積もる |
| 8 | フレームループ統合 | 全体が連続的に動く |

**Phase 1 完了基準：** 2,000 枚で 60fps 以上、3 レイヤー分離が完了していること。

### Phase 2: FALLING を Wasm 化

| Step | 内容 | 確認方法 |
|---|---|---|
| 1 | `Cargo.toml` + `src/lib.rs` を作成 | `wasm-pack build` が通る |
| 2 | SoA バッファを Wasm メモリ上に確保 | JS からポインタ経由で読める |
| 3 | `seedPetals()` で初期データ生成 | ON_TREE 花びらが正しい位置に表示 |
| 4 | `activateReadyPetals()` 実装 | 花びらが 1 枚ずつ FALLING に遷移 |
| 5 | `stepFalling()` 実装 | 物理演算が Wasm で動く |
| 6 | `landedIndices` でイベント取得 → Layer C に書き込み | 着地が正しく動作 |
| 7 | Layer B の partial upload | active 範囲だけ GPU 転送 |

**Phase 2 完了基準：** 10,000 枚で 60fps 以上。
Phase 2 完了前は、公開 UI の上限を安全側に絞る。

### Phase 3: 最適化

- `activate_ready_petals` の走査を ON_TREE リストに限定
- Layer B の `bufferAttribute.updateRange` で転送量を最小化
- `InstancedBufferGeometry` の attribute 直接操作
- 不要な `dummy.updateMatrix()` + `setMatrixAt()` の完全排除

**Phase 3 完了基準：** 50,000 枚以上で破綻しないことを検証。

### Phase 4（将来）

- 本当に数十万を狙うなら、FALLING も GPU コンピュートシェーダー（WebGPU TSL）に寄せる
- Wasm は event / scheduling / spawn 管理に限定
- 10 万枚以上を目標

---

## 14. 禁止事項

| 禁止 | 理由 |
|---|---|
| 花びらに個別 `new THREE.Mesh` | DrawCall 爆発 |
| 毎フレーム `new Vector3()` 等のオブジェクト生成 | GC カクつき |
| AoS のまま Wasm に渡す | 恩恵なし。SoA 必須 |
| ON_TREE / ON_GROUND を毎フレーム CPU 更新 | 静的なものは触らない |
| 全花びらの毎フレーム `setMatrixAt()` | CPU ボトルネック |
| 外部物理ライブラリ（cannon.js 等） | 自前 Wasm で十分 |
| `mrdoob/three.wasm` の使用 | エイプリルフールのジョーク |
| 花びらのリサイクル・リスポーン | コンセプト違反 |
| 花房オブジェクト（ピンク球体 Mesh）生成 | ON_TREE 花びらが満開を表現 |
| `resetPetal()` 関数の作成 | リサイクル処理は存在しない |

---

## 15. 完成後の確認チェックリスト

| # | 確認項目 | 期待値 |
|---|---|---|
| 1 | 起動直後 | 木全体がピンク花びらで覆われ「満開」 |
| 2 | 3秒後〜 | 花びらが1枚ずつランダムに散り始める |
| 3 | 落下の動き | 風に揺れ、ひらひら回転して落ちる |
| 4 | 着地後 | 地面に永久残留。消えない |
| 5 | 島の上 | 花びらが草の上に積もる（`GROUND_Y`） |
| 6 | 島の外 | 花びらが低い草原に着地（`GROUND_Y_OUTER`） |
| 7 | 60秒後 | 地面に花びら蓄積、木の花が減少 |
| 8 | 全散後 | 木が裸、地面が花びらで覆われる |
| 9 | ON_TREE | GPU 揺れのみ。CPU 更新なし |
| 10 | FALLING | Wasm で処理（Phase 2+）。active 範囲のみ更新 |
| 11 | ON_GROUND | 静的。毎フレーム更新なし |
| 12 | fps (Phase 1) | 2,000枚で 60fps以上 |
| 13 | fps (Phase 2) | 10,000枚で 60fps以上 |
| 14 | 背景 | スカイブルー + フォグフェードアウト |
| 15 | 島 | こんもり円形、側面が土色 |
| 16 | 木 | proctree.js 自然な枝分かれ |
| 17 | 花びら形状 | GLSL で花びら型カット（四角くない） |
| 18 | 操作 | ドラッグ回転、ホイールズーム |
| 19 | 自動回転 | ゆっくり自動回転 |
| 20 | 3レイヤー | Tree / Falling / Ground が分離されている |

---

## 16. 起動方法

```bash
# Phase 1（JS のみ — Wasm なし）
cd sakura-demo
npx serve .

# Phase 2 以降（Wasm ビルドが必要）
cd sakura-demo
wasm-pack build --target web --release
npx serve .

# → http://localhost:3000
```

---

*以上 — AIエージェント向け最終実装指示書（Rust + Wasm ルート）*
