# 🌸 桜の花びら物理演算デモ — AIエージェント向け最終実装指示書

**Version FINAL（v1〜v3 統合版）| 2026年4月**

> **この指示書について**
> 本ドキュメントは初期仕様書（v1）、修正指示書（v2）、コンセプト修正指示書（v3）の3つを統合し、矛盾を解消した**唯一の正**となる最終実装指示書です。
> 過去の仕様書は参照不要です。本書だけに従ってください。

---

## 0. 最重要コンセプト（実装前に必ず理解すること）

```
【花びらの一生 — リサイクルしない】

① ON_TREE   ：木の枝（花房位置）に咲いている ← 初期状態。木が満開に見える
      ↓ ランダムな待機時間の経過後、1枚ずつ離脱
② FALLING   ：空中を舞いながら落下する       ← 風・重力・回転の物理演算
      ↓ 地面に到達
③ ON_GROUND ：地面に静止する                 ← 永久にその場に残る。消えない
```

- 花びらの総数は **PETAL_COUNT = 2000（固定）**
- 起動直後は全枚数が「木に咲いている状態（ON_TREE）」で表示される → **満開に見える**
- 時間とともに1枚ずつランダムに散り始め、地面に積もっていく
- 全部散ると木が裸になり、地面が花びらで覆われた状態になる
- **リサイクル・リスポーンは一切しない**
- `resetPetal()` のようなリサイクル関数は作らない

---

## 1. プロジェクト概要

### 1-1. 目的

ローポリスタイルの桜の木から花びらが物理演算に従ってはらはらと落下するインタラクティブな3Dデモをブラウザ上に実装する。花びら1枚1枚が独立した状態を持ち、「満開→散り始め→散り終わり」の自然な流れをリアルタイムに表現する。

### 1-2. 参照デザイン

添付画像を参照デザインとする。主な特徴：

- ローポリ（low-poly）スタイルの3D
- 木：太い幹から自然に枝分かれし、枝全体が白〜淡ピンクの花びらで覆われている
- 幹：茶色、表面にテクスチャ感があり、根元が少し広がっている
- 花の密度：非常に高密度。枝が見えないほど花びらで覆われている
- 背景：淡いピンク系の単色グラデーション
- 全体の雰囲気：柔らかい、メルヘン、かわいい

### 1-3. 技術スタック

| 項目 | 採用技術 | 備考 |
|---|---|---|
| 3Dレンダリング | three.js r160 | CDN直接読み込み |
| 木の生成 | proctree.js | L-systemベースのプロシージャル木生成 |
| 言語 | JavaScript（ES Module） | TypeScript不使用 |
| ビルドツール | なし | index.html 単体で動作 |
| 物理演算 | 自前実装 | 外部物理ライブラリ不使用 |
| 花びら描画 | InstancedMesh + GLSL ShaderMaterial | 個別Mesh禁止 |

---

## 2. ファイル構成

```
sakura-demo/
└── index.html    ← すべてのコードをこの1ファイルに集約
```

外部画像・フォント・CSS・JSファイルは一切使わない。

### CDN設定（head内に記述）

```html
<!-- proctree.js -->
<script src="https://cdn.jsdelivr.net/gh/supereggbert/proctree.js/proctree.js"></script>

<!-- three.js (importmap) -->
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

---

## 3. パラメータ定数（ファイル先頭に定義）

```javascript
const PETAL_COUNT    = 2000;   // 花びら総数（固定・リサイクルなし）
const PETAL_SIZE     = 0.10;   // 花びらサイズ
const GROUND_Y       = 0.32;   // 島上面の着地Y座標
const GROUND_Y_OUTER = -0.20;  // 島の外の着地Y座標
const ISLAND_RADIUS  = 3.2;    // 島の半径（着地判定に使用）
const AUTO_ROTATE_SPEED = 0.3; // 自動回転速度

const PARAMS = {
  windStrength:  0.002,
  gravity:       0.0006,
  airResistance: 0.97,
};

const PETAL_STATE = {
  ON_TREE:   0,  // 木に咲いている
  FALLING:   1,  // 落下中
  ON_GROUND: 2,  // 地面に静止（永久残留）
};

const BLOSSOM_POSITIONS = []; // 花房（枝先）の座標配列。createTree() で収集
```

---

## 4. デザイン仕様

### 4-1. カラーパレット

| 要素 | カラーコード | 用途 |
|---|---|---|
| 背景 | `#87CEEB` | スカイブルー |
| フォグ | `#B0D8F0` | 遠景のフェードアウト用 |
| 桜の花びら | `#F0B0C8` | シェーダーの基本色（インスタンスごとに微変化） |
| 幹・枝 | `#3D1C0F` | 濃い茶色 |
| 草（島上面） | `#7EC850` | 明るいグリーン |
| 土（島側面） | `#5C3A1E` | 茶色（v1の黒ではなく土色） |
| 背景草原 | `#5A9E35` | 島より少し暗いグリーン |
| 岩（明） | `#D0D0D0` | ライトグレー |
| 岩（暗） | `#909090` | ダークグレー |

### 4-2. ライティング

| ライト種類 | 設定値 | 役割 |
|---|---|---|
| `AmbientLight` | color: `#FFF8F0`, intensity: 0.8 | 全体を柔らかく照らす |
| `DirectionalLight` | color: `#FFFFFF`, intensity: 1.2, pos: (5, 10, 5) | 主光源 |
| `DirectionalLight`（補助） | color: `#C8E8FF`, intensity: 0.4, pos: (-3, 5, -3) | 反射光 |

影（Shadow）は無効。

### 4-3. カメラ・構図

| 項目 | 値 |
|---|---|
| カメラ | `PerspectiveCamera`（fov: 45, near: 0.1, far: 100） |
| 初期位置 | `position(0, 4, 8)` → `lookAt(0, 1.5, 0)` |
| コントロール | `OrbitControls`（ドラッグ回転、ホイールズーム） |
| 自動回転 | `autoRotate: true`, `autoRotateSpeed: 0.3` |
| ズーム範囲 | `minDistance: 3`, `maxDistance: 15` |
| 慣性 | `enableDamping: true`, `dampingFactor: 0.05` |
| ターゲット | `controls.target.set(0, 1.5, 0)` |

---

## 5. 各オブジェクトの実装仕様

### 5-1. 地面（島）— `createGround()`

参照画像のような「こんもりした丸い島」を表現する。3つのパーツで構成：

**① 島の上面（草）**
- `CylinderGeometry(3.2, 3.2, 0.02, 10)` — 薄い円盤
- マテリアル：`MeshLambertMaterial({ color: 0x7EC850 })`
- `position.y = 0.31`

**② 島の本体（側面が土色）**
- `CylinderGeometry(3.2, 3.8, 0.6, 10)` — 上が狭く下が広い台形
- 頂点をランダムにずらしてローポリ感を出す（X, Z 方向に ±0.15）
- マルチマテリアル：`[soilMat, grassMat, soilMat]`
  - `soilMat`：`color: 0x5C3A1E`（土色）→ 側面と底面
  - `grassMat`：`color: 0x7EC850`（草色）→ 上面キャップ
- `position.y = 0.01`

> ⚠️ `CylinderGeometry` のマテリアルグループ順序は `0=側面, 1=上面cap, 2=下面cap`。配列の順番を間違えないこと。

**③ 背景の草原**
- `PlaneGeometry(120, 120, 12, 12)` — 広大な平面
- 頂点の Z をランダムオフセット（±0.4）して平坦すぎないようにする
- マテリアル：`MeshLambertMaterial({ color: 0x5A9E35 })`（島より暗い）
- `rotation.x = -Math.PI / 2`, `position.y = -0.20`

### 5-2. 木 — `createTree()`

**proctree.js を使用**して自然な枝分かれ構造を生成する。

**proctree.js パラメータ：**
```javascript
{
  clumpMax: 0.454,
  clumpMin: 0.404,
  lengthFalloffFactor: 0.85,
  lengthFalloffPower: 0.99,
  branchFactor: 2.8,
  radiusFalloffRate: 0.6,
  climbRate: 0.5,
  trunkKink: 0.09,
  maxRadius: 0.12,
  treeSteps: 5,
  taperRate: 0.95,
  twistRate: 3.02,
  segments: 6,
  levels: 4,
  sweepAmount: 0,
  initialBranchLength: 0.85,
  trunkLength: 1.8,
  dropAmount: -0.1,
  growAmount: 0.235,
  vMultiplier: 0.36,
  twigScale: 0.35,
  seed: 262
}
```

**幹・枝メッシュの生成手順：**
1. `new Tree(treeParams)` でツリーオブジェクトを生成
2. `tree.verts` → `Float32BufferAttribute` に変換、`tree.faces` → インデックスに変換
3. `BufferGeometry` を作成し、`computeVertexNormals()`
4. `MeshLambertMaterial({ color: 0x3D1C0F })` で茶色のメッシュとして `scene.add`
5. `trunkMesh.position.y = 0.3`（地面から少し上に）

**花房位置の収集（重要）：**
- `tree.twigVerts` を3つおきにサンプリングして `BLOSSOM_POSITIONS` 配列に追加
- Y座標は `+0.3` オフセット（trunkMeshのオフセット分）
- 密度を上げるために、各花房位置の周囲にランダムで3点追加する

```javascript
// 枝先位置の収集（花びらの初期配置座標に使う）
for (let i = 0; i < tree.twigVerts.length; i += 3) {
  BLOSSOM_POSITIONS.push(new THREE.Vector3(
    tree.twigVerts[i].x,
    tree.twigVerts[i].y + 0.3,
    tree.twigVerts[i].z
  ));
}

// 密度補完：各位置の周囲にランダム3点追加
const extra = [];
for (const pos of BLOSSOM_POSITIONS) {
  for (let j = 0; j < 3; j++) {
    extra.push(new THREE.Vector3(
      pos.x + (Math.random() - 0.5) * 0.4,
      pos.y + (Math.random() - 0.5) * 0.2,
      pos.z + (Math.random() - 0.5) * 0.4
    ));
  }
}
BLOSSOM_POSITIONS.push(...extra);
```

> ⚠️ **花房オブジェクト（ピンクの球体）は一切生成しない。** ON_TREE 状態の花びら自体が「咲いている花」を表現する。大きなピンクの玉を作ると不自然になる。

### 5-3. 岩 — `createRocks()`

島の中央付近に小さめのローポリ岩を4個配置する。

**各岩の設定値：**
```javascript
const configs = [
  { x:  0.8, z:  0.5, sx: 0.35, sy: 0.28, sz: 0.32, ry: 0.3 },
  { x: -0.5, z:  0.8, sx: 0.28, sy: 0.22, sz: 0.30, ry: 1.1 },
  { x:  0.3, z: -0.7, sx: 0.42, sy: 0.25, sz: 0.35, ry: 2.2 },
  { x: -0.9, z: -0.3, sx: 0.20, sy: 0.18, sz: 0.22, ry: 0.8 },
];
```

**各岩の生成手順：**
1. `IcosahedronGeometry(0.5, 1)` で基本形を生成
2. 頂点をランダムにずらす（X, Z: ±0.15 / Y: ±0.10）
3. `computeVertexNormals()`
4. 色：明度 0.65〜0.85 のグレーをランダム生成
5. `position.set(cfg.x, 0.28 + cfg.sy * 0.3, cfg.z)`
6. `rotation.y = cfg.ry`
7. `scale.set(cfg.sx, cfg.sy, cfg.sz)`

### 5-4. 花びら — `createPetals()`

**InstancedMesh + ShaderMaterial** で全花びらを1つの DrawCall で描画する。

**① シェーダーマテリアル（花びら型にカット）**

```glsl
// --- 頂点シェーダー ---
varying vec2 vUv;
varying float vBrightness;
void main() {
  vUv = uv;
  // インスタンスごとにわずかに明るさを変える
  vBrightness = 0.9 + fract(sin(float(gl_InstanceID) * 12.9898) * 43758.5453) * 0.2;
  gl_Position = projectionMatrix * modelViewMatrix * instanceMatrix * vec4(position, 1.0);
}

// --- フラグメントシェーダー ---
uniform vec3 color;
varying vec2 vUv;
varying float vBrightness;

void main() {
  vec2 uv = vUv - 0.5;

  // 花びら形状：3つの円のunion
  float d1 = length(uv - vec2(-0.18, 0.05)) - 0.28; // 左
  float d2 = length(uv - vec2( 0.18, 0.05)) - 0.28; // 右
  float d3 = length(uv - vec2( 0.0, -0.15)) - 0.22; // 下
  float d = min(min(d1, d2), d3);

  float alpha = 1.0 - smoothstep(-0.02, 0.02, d);
  if (alpha < 0.1) discard; // 透明部分はピクセルを捨てる

  vec3 c = color * vBrightness;
  c += (1.0 - length(uv) * 2.0) * 0.08; // 中心ほど白
  gl_FragColor = vec4(c, alpha);
}
```

**マテリアル設定：**
- `uniforms.color`: `new THREE.Color(0xF0B0C8)`
- `transparent: true`
- `side: THREE.DoubleSide`
- `depthWrite: false`（透明マテリアルのZファイティング対策）

**② ジオメトリ**
- `PlaneGeometry(PETAL_SIZE * 1.5, PETAL_SIZE)` — 横長の板

**③ InstancedMesh**
- `new THREE.InstancedMesh(petalGeo, petalMaterial, PETAL_COUNT)`
- `instanceMatrix.setUsage(THREE.DynamicDrawUsage)`

> ⚠️ 個別の `new THREE.Mesh` を花びら枚数分作らないこと。DrawCallが爆発する。

**④ 花びらデータ配列の初期化**

全花びらを ON_TREE 状態で初期化する：

```javascript
petalData = [];
for (let i = 0; i < PETAL_COUNT; i++) {
  const origin = BLOSSOM_POSITIONS[Math.floor(Math.random() * BLOSSOM_POSITIONS.length)];

  petalData.push({
    state: PETAL_STATE.ON_TREE,

    treePosition: new THREE.Vector3(
      origin.x + (Math.random() - 0.5) * 0.5,
      origin.y + (Math.random() - 0.5) * 0.3,
      origin.z + (Math.random() - 0.5) * 0.5
    ),

    position:      new THREE.Vector3(),
    velocity:      new THREE.Vector3(),
    rotation:      new THREE.Euler(rand2PI, rand2PI, rand2PI),
    rotationSpeed: new THREE.Vector3(rand004, rand002, rand004),
    windPhase:     Math.random() * Math.PI * 2,

    // 散り始めるまでの待機フレーム（最初3秒は静止 → 60秒かけて全部散る）
    fallDelay: 180 + Math.floor(Math.random() * 3600),
  });

  // 初期表示：木に咲いている位置にMatrixをセット
  dummy.position.copy(petalData[i].treePosition);
  dummy.rotation.copy(petalData[i].rotation);
  dummy.scale.setScalar(1.0);
  dummy.updateMatrix();
  petalMesh.setMatrixAt(i, dummy.matrix);
}
petalMesh.instanceMatrix.needsUpdate = true;
```

---

## 6. 物理演算仕様 — `updatePetals(time)`

毎フレーム呼び出す。`frameCount` をグローバルでインクリメントする。
`dummy` は `new THREE.Object3D()` でグローバルに1つだけ生成し使い回す。

```
各花びらについて：

【ON_TREE の場合】
  - frameCount >= fallDelay なら → FALLING に遷移、treePosition から落下開始
  - まだ待機中なら → treePosition に微風揺れを加えて表示
    （sin/cos で X,Y,Z をそれぞれ ±0.01 程度ゆらす）

【ON_GROUND の場合】
  - 何もしない。Matrix もそのまま。永久残留。

【FALLING の場合】
  1. 重力：velocity.y -= PARAMS.gravity
  2. 風（複数周波数を重ねる）：
     wind = sin(t*0.8 + phase) * ws
          + sin(t*1.7 + phase*1.3) * ws * 0.4
          + sin(t*0.3 + phase*0.7) * ws * 0.6
     velocity.x += wind
     velocity.z += cos(t*0.6 + phase) * ws * 0.8
  3. 空気抵抗：velocity.multiplyScalar(PARAMS.airResistance)
  4. 位置更新：position.add(velocity)
  5. 回転更新：rotation.x += rotationSpeed.x, rotation.z += rotationSpeed.z
  6. 着地判定：
     distFromCenter = sqrt(x² + z²)
     groundLevel = distFromCenter < ISLAND_RADIUS ? GROUND_Y : GROUND_Y_OUTER
     if (position.y <= groundLevel) → ON_GROUND に遷移
       - position.y = groundLevel + random * 0.01
       - velocity = (0, 0, 0)
       - rotationSpeed = (0, 0, 0)
       - rotation を水平に寝かせる（x ≈ -π/2, y = random, z ≈ 0）
  7. Matrix 更新：dummy に position/rotation/scale をセット → setMatrixAt

ループ後：petalMesh.instanceMatrix.needsUpdate = true
```

> ⚠️ 毎フレーム `new Vector3()` などで新規オブジェクトを生成しないこと → GCの原因。
> ⚠️ `resetPetal()` 関数は存在しない。リサイクル処理は一切なし。

---

## 7. シーン初期化 — `initScene()`

```javascript
scene = new THREE.Scene();
scene.background = new THREE.Color(0x87CEEB);
scene.fog = new THREE.Fog(0xB0D8F0, 30, 80); // 遠景フェードアウト

camera = new THREE.PerspectiveCamera(45, innerWidth / innerHeight, 0.1, 100);
camera.position.set(0, 4, 8);
camera.lookAt(0, 1.5, 0);

renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(innerWidth, innerHeight);
renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
renderer.setClearColor(0x87CEEB);
document.body.appendChild(renderer.domElement);

controls = new OrbitControls(camera, renderer.domElement);
controls.autoRotate = true;
controls.autoRotateSpeed = 0.3;
controls.enableDamping = true;
controls.dampingFactor = 0.05;
controls.minDistance = 3;
controls.maxDistance = 15;
controls.target.set(0, 1.5, 0);

// ライティング（セクション4-2参照）を追加
```

---

## 8. アニメーションループ — `animate()`

```javascript
function animate() {
  requestAnimationFrame(animate);
  const time = performance.now() * 0.001;
  controls.update();   // enableDamping のために毎フレーム必須
  updatePetals(time);
  renderer.render(scene, camera);
}
```

---

## 9. その他

### リサイズ対応 — `onWindowResize()`
- `camera.aspect` と `renderer.setSize` を更新
- `window.addEventListener('resize', onWindowResize)` を `initScene()` 内で登録

### FPS表示（任意）
- 画面左上に `fps | 木: N 落下中: N 地面: N` をリアルタイム表示すると動作確認に便利
- 本番で不要なら削除してよい

### HTML/CSS
- `body { margin: 0; overflow: hidden; background: #87CEEB; }`
- `canvas { display: block; }`
- 操作ヒント：画面下部に `ドラッグで回転 ・ スクロールでズーム` と小さく表示

---

## 10. 実装手順（この順番で実装すること）

| Step | 関数 | 内容 | 完了の確認方法 |
|---|---|---|---|
| 1 | `initScene()` | Scene / Camera / Renderer / Controls / Lights | スカイブルーの画面が表示される |
| 2 | `createGround()` | 島（上面＋本体＋背景草原） | 緑の島と草原が見える、側面が茶色 |
| 3 | `createTree()` | proctree.js で幹・枝を生成、BLOSSOM_POSITIONS を収集 | 枝分かれした茶色の木が立っている |
| 4 | `createRocks()` | 岩4個を島の上に配置 | 灰色のローポリ岩がある |
| 5 | `createPetals()` | InstancedMesh + ShaderMaterial、全花びら ON_TREE で初期化 | 木全体がピンクの花びらで覆われ「満開」に見える |
| 6 | `updatePetals()` | 物理演算ループ | 花びらが1枚ずつ散り始める |
| 7 | `animate()` | requestAnimationFrame のメインループ | 全体が連続的に動く |
| 8 | 仕上げ | リサイズ対応・FPS表示・操作ヒント | ウィンドウリサイズで正常動作 |

---

## 11. 禁止事項

| 禁止 | 理由 |
|---|---|
| 個別の `new THREE.Mesh` を花びら枚数分生成 | DrawCall 爆発 → 必ず `InstancedMesh` を使う |
| 毎フレーム `new Vector3()` 等で新規オブジェクト生成 | GC によるカクつき → 使い回す |
| 外部物理ライブラリ（cannon.js, ammo.js 等） | 自前実装で十分 |
| `async/await` で外部リソース読み込み | 完全オフライン動作を維持（CDN除く） |
| TypeScript やバンドラー（Vite 等） | index.html 単体完結 |
| 花びらのリサイクル・リスポーン | コンセプト違反。花びらは有限・使い捨て |
| 花房オブジェクト（ピンクの球体 Mesh）の生成 | ON_TREE 花びらが「咲いている花」を表現する |
| `file://` での直接閲覧 | CORS エラー → `npx serve .` 等でローカルサーバー使用 |

---

## 12. 完成後の確認チェックリスト

| # | 確認項目 | 期待値 |
|---|---|---|
| 1 | 起動直後 | 木全体がピンクの花びらで覆われ「満開」に見える |
| 2 | 3秒後〜 | 花びらが1枚ずつランダムなタイミングで散り始める |
| 3 | 落下の動き | 風に揺れながら、ひらひらと回転して落ちる |
| 4 | 着地後 | 地面にそのまま残る。消えない。リスポーンしない |
| 5 | 島の上 | 花びらが島の草の上に積もる |
| 6 | 島の外 | 花びらが草原の低い位置に着地する |
| 7 | 60秒後 | 地面に花びらが大量に積もっている。木の花が減っている |
| 8 | 全部散った後 | 木が裸になり、地面が花びらで覆われた状態 |
| 9 | 背景 | スカイブルー + 遠景がフォグでぼんやりフェードアウト |
| 10 | 島の形 | こんもりした円形、側面が土色（茶色） |
| 11 | 木の形 | proctree.js による自然な枝分かれ |
| 12 | 花びらの形 | 四角い板ではなく花びら型（GLSLシェーダー） |
| 13 | 岩 | 4個のグレーのローポリ岩が島の上にある |
| 14 | fps | 2000枚で60fps以上 |
| 15 | 操作 | ドラッグで自由に視点回転、ホイールでズーム |
| 16 | 自動回転 | ゆっくりとシーンが自動回転する |
| 17 | レスポンシブ | ウィンドウリサイズで正常に対応 |

---

## 13. 起動方法

```bash
cd sakura-demo
npx serve .
# → http://localhost:3000 をブラウザで開く
```

---

## 14. 将来的な改善アイデア（任意）

以下は本指示書のスコープ外。余力があれば検討：

- 散り始めのトリガー：クリックで散り始める演出
- 花びらの色グラデーション：木の高い位置ほど白、低い位置ほど濃いピンク
- 着地時のバウンド：`velocity.y = 0.01` で小さく跳ねてから静止
- Tweakpane UI：風・重力・散る速さをスライダーでリアルタイム調整
- three.wasm 移行：`updatePetals()` の中身をWasm呼び出しに置き換えるだけで済む設計になっている

---

*以上 — AIエージェント向け最終実装指示書*
