# 🌸 桜デモ 修正指示書
## 現状からの差分改善仕様
**Version 2.0 | 2026年4月**

> この指示書は現在の実装（v1）の問題点を列挙し、優先順位付きで修正方法を指示するものです。
> 新規ファイルを作るのではなく、**既存の index.html を以下の指示に従って上書き修正**してください。

---

## 📸 現状の問題点（ビフォー診断）

現在のスクリーンショットと参照デザインを比較した際の問題点を優先度付きで列挙する。

| 優先度 | 問題箇所 | 現状 | 目標 |
|---|---|---|---|
| 🔴 高 | 地面の形状 | 平たいリング状（ドーナツ型） | こんもりした円形の島（断面が見える丘） |
| 🔴 高 | 木の構造 | 枝なしの太い柱1本 | 幹＋主枝＋小枝が自然に分岐 |
| 🔴 高 | 花びらの形 | 正方形の板（四角く見える） | 花びら型（GLSLシェーダーで整形） |
| 🟡 中 | 花房の配置 | 空中に浮いているだけ | 枝の先端に自然にくっついている |
| 🟡 中 | 地面の色 | 暗い茶色 | 明るいグリーン `#7EC850`（参照画像通り） |
| 🟡 中 | 背景 | グラデーション | 単色スカイブルー `#87CEEB` |
| 🟢 低 | パラメータUI | なし | Tweakpaneで風・枚数をリアルタイム調整 |
| 🟢 低 | 花びらの散布範囲 | 地面外にも散乱 | 島の上と周囲に自然に着地 |

---

## 🛠 修正1：木の構造をproctree.jsで再生成【優先度：🔴 高】

### 問題
現在は `CylinderGeometry` 1本のただの柱。枝がなく、花房が宙に浮いて見える。

### 解決策
**proctree.js**（L-systemベースのプロシージャル木生成ライブラリ）を使って枝分かれ構造を自動生成する。

### CDN追加

```html
<!-- headタグ内に追加 -->
<script src="https://cdn.jsdelivr.net/gh/supereggbert/proctree.js/proctree.js"></script>
```

### createTree() 関数を以下に丸ごと置き換える

```javascript
function createTree() {
  // proctree.js のパラメータ設定
  const treeParams = {
    clumpMax: 0.454,
    clumpMin: 0.404,
    lengthFalloffFactor: 0.85,
    lengthFalloffPower: 0.99,
    branchFactor: 2.8,        // 枝分かれの多さ（増やすと桜っぽく）
    radiusFalloffRate: 0.6,
    climbRate: 0.5,
    trunkKink: 0.09,
    maxRadius: 0.12,
    treeSteps: 5,              // 分岐の深さ（5〜7が桜らしい）
    taperRate: 0.95,
    twistRate: 3.02,
    segments: 6,
    levels: 4,
    sweepAmount: 0,
    initialBranchLength: 0.85,
    trunkLength: 1.8,          // 幹の長さ
    dropAmount: -0.1,
    growAmount: 0.235,
    vMultiplier: 0.36,
    twigScale: 0.35,
    seed: 262                  // シードを変えると形が変わる（262が桜っぽい）
  };

  const tree = new Tree(treeParams);

  // --- 幹・枝のジオメトリ ---
  const trunkGeo = new THREE.BufferGeometry();
  const verts = [];
  const indices = [];

  for (let i = 0; i < tree.verts.length; i++) {
    verts.push(tree.verts[i].x, tree.verts[i].y, tree.verts[i].z);
  }
  for (let i = 0; i < tree.faces.length; i++) {
    indices.push(tree.faces[i][0], tree.faces[i][1], tree.faces[i][2]);
  }

  trunkGeo.setAttribute('position', new THREE.Float32BufferAttribute(verts, 3));
  trunkGeo.setIndex(indices);
  trunkGeo.computeVertexNormals();

  const trunkMat = new THREE.MeshLambertMaterial({ color: 0x3D1C0F });
  const trunkMesh = new THREE.Mesh(trunkGeo, trunkMat);
  trunkMesh.position.y = 0.3; // 地面から少し上に
  scene.add(trunkMesh);

  // --- 枝先の位置を収集して花房を配置 ---
  // twigs（枝先）の位置を使って花房をくっつける
  const twigPositions = [];
  for (let i = 0; i < tree.twigVerts.length; i++) {
    twigPositions.push(new THREE.Vector3(
      tree.twigVerts[i].x,
      tree.twigVerts[i].y + 0.3,
      tree.twigVerts[i].z
    ));
  }

  // 花房をtwigの位置に配置（間引いて配置）
  const blossomMat = new THREE.MeshLambertMaterial({
    color: 0xE8709A,
    side: THREE.DoubleSide
  });

  // twigVertsをクラスタリングして花房を配置（全部につけると多すぎる）
  const placedPositions = new Set();
  let blossomCount = 0;
  for (let i = 0; i < twigPositions.length; i += 6) { // 6個おきに1つ
    if (blossomCount > 40) break; // 最大40個
    const pos = twigPositions[i];
    const key = `${Math.round(pos.x*2)},${Math.round(pos.y*2)},${Math.round(pos.z*2)}`;
    if (placedPositions.has(key)) continue;
    placedPositions.add(key);

    const size = 0.35 + Math.random() * 0.25;
    const blossomGeo = new THREE.SphereGeometry(size, 5, 4); // ローポリ
    // 頂点をわずかにランダムにずらしてさらにローポリ感を出す
    const posAttr = blossomGeo.attributes.position;
    for (let v = 0; v < posAttr.count; v++) {
      posAttr.setX(v, posAttr.getX(v) + (Math.random() - 0.5) * 0.1);
      posAttr.setY(v, posAttr.getY(v) + (Math.random() - 0.5) * 0.1);
      posAttr.setZ(v, posAttr.getZ(v) + (Math.random() - 0.5) * 0.1);
    }
    blossomGeo.computeVertexNormals();

    // 色をわずかにランダムに変化させる
    const hue = 0.93 + Math.random() * 0.04;
    const blossomColor = new THREE.Color().setHSL(hue, 0.7, 0.72);
    const blossom = new THREE.Mesh(blossomGeo, new THREE.MeshLambertMaterial({
      color: blossomColor
    }));
    blossom.position.copy(pos);
    scene.add(blossom);

    // 花房の位置を記録（花びらのリスポーン座標に使う）
    BLOSSOM_POSITIONS.push(pos.clone());
    blossomCount++;
  }
}

// 花びらリスポーン用にグローバルで保持
const BLOSSOM_POSITIONS = [];
```

> 💡 `seed` の値を変えるだけで木の形が変わる。`262` の他に `100`・`512`・`777` あたりも試してみること。

---

## 🛠 修正2：地面を「こんもりした島」に修正【優先度：🔴 高】

### 問題
現在はリング状（ドーナツ型）に見える。参照デザインは断面が見えるこんもりした丘。

### createGround() 関数を以下に丸ごと置き換える

```javascript
function createGround() {
  // --- メインの島（上面） ---
  // radialSegments を低くしてローポリ八角形に
  const islandGeo = new THREE.CylinderGeometry(3.5, 4.0, 0.6, 8);

  // 頂点をわずかにランダムオフセットしてローポリ感を強調
  const posAttr = islandGeo.attributes.position;
  for (let i = 0; i < posAttr.count; i++) {
    const y = posAttr.getY(i);
    if (y > 0) { // 上面の頂点のみ
      posAttr.setX(i, posAttr.getX(i) + (Math.random() - 0.5) * 0.2);
      posAttr.setZ(i, posAttr.getZ(i) + (Math.random() - 0.5) * 0.2);
      posAttr.setY(i, y + Math.random() * 0.05);
    }
  }
  islandGeo.computeVertexNormals();

  // 上面と側面で別々のマテリアルを使う（マルチマテリアル）
  const grassMat = new THREE.MeshLambertMaterial({ color: 0x7EC850 }); // 明るいグリーン
  const sideMat  = new THREE.MeshLambertMaterial({ color: 0x1A1A1A }); // ほぼ黒

  // CylinderGeometry のグループ: 0=側面, 1=上面cap, 2=下面cap
  const island = new THREE.Mesh(islandGeo, [sideMat, grassMat, sideMat]);
  island.position.y = 0;
  scene.add(island);

  // --- 広い地面（背景の草原） ---
  const groundGeo = new THREE.PlaneGeometry(60, 60, 8, 8);
  // 頂点をわずかにランダムにY方向オフセット
  const groundPos = groundGeo.attributes.position;
  for (let i = 0; i < groundPos.count; i++) {
    groundPos.setZ(i, groundPos.getZ(i) + (Math.random() - 0.5) * 0.3);
  }
  groundGeo.computeVertexNormals();

  const groundMat = new THREE.MeshLambertMaterial({ color: 0x6AAA40 }); // やや暗いグリーン
  const ground = new THREE.Mesh(groundGeo, groundMat);
  ground.rotation.x = -Math.PI / 2;
  ground.position.y = -0.31; // 島の底面と揃える
  scene.add(ground);
}
```

> ⚠️ `CylinderGeometry` のグループインデックスは `0=側面, 1=上面, 2=下面` の順。マルチマテリアルの配列の順番を間違えないこと。

---

## 🛠 修正3：花びらをGLSLシェーダーで花びら型に【優先度：🔴 高】

### 問題
現在の `PlaneGeometry` + `MeshBasicMaterial` では四角い板にしか見えない。

### 解決策
`ShaderMaterial` を使い、フラグメントシェーダーで花びら形状以外を透明にする。

### createPetals() のマテリアル部分を以下に置き換える

```javascript
// ShaderMaterialで花びら型に
const petalMaterial = new THREE.ShaderMaterial({
  uniforms: {
    color: { value: new THREE.Color(0xF0B0C8) }
  },
  vertexShader: `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  fragmentShader: `
    uniform vec3 color;
    varying vec2 vUv;

    void main() {
      // UV座標を中心基準に変換
      vec2 uv = vUv - 0.5;

      // 花びら形状：上半分に2つの円を組み合わせたハート型
      float d1 = length(uv - vec2(-0.18, 0.05)) - 0.28; // 左の丸
      float d2 = length(uv - vec2( 0.18, 0.05)) - 0.28; // 右の丸
      float d3 = length(uv - vec2( 0.0, -0.15)) - 0.22; // 下の丸み

      // 3つの円の合成（最小距離＝union）
      float d = min(min(d1, d2), d3);

      // エッジを少しぼかして自然な見た目に
      float alpha = 1.0 - smoothstep(-0.02, 0.02, d);

      if (alpha < 0.1) discard; // 完全透明部分はピクセルを捨てる

      // 中心から外側に向かって少し明るくする
      float brightness = 1.0 + length(uv) * 0.3;
      gl_FragColor = vec4(color * brightness, alpha);
    }
  `,
  transparent: true,
  side: THREE.DoubleSide,
  depthWrite: false // 透明マテリアルのZファイティング対策
});

// InstancedMesh に適用
const petalGeo = new THREE.PlaneGeometry(PETAL_SIZE * 1.5, PETAL_SIZE);
const petalMesh = new THREE.InstancedMesh(petalGeo, petalMaterial, PETAL_COUNT);
petalMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
scene.add(petalMesh);
```

> 💡 `discard` を使うことで透明部分のピクセルを完全に破棄できる。`alphaTest` より綺麗に見える。

---

## 🛠 修正4：背景を単色スカイブルーに【優先度：🟡 中】

### 問題
現在は背景にグラデーションや霧がかかっており、参照デザインの単色スカイブルーと異なる。

### initScene() 内の背景設定を以下に置き換える

```javascript
// 背景色を単色に設定
renderer.setClearColor(0x87CEEB); // スカイブルー

// scene.background や scene.fog が設定されている場合は削除
scene.background = null;
scene.fog = null;
```

---

## 🛠 修正5：花びらのリスポーン位置を花房に連動【優先度：🟡 中】

### 問題
花びらが木と関係ない位置から生まれている。花房（blossom）の位置からリスポーンすべき。

### resetPetal() 関数を以下に置き換える

```javascript
function resetPetal(petal) {
  // BLOSSOM_POSITIONS が空の場合はフォールバック
  const origins = BLOSSOM_POSITIONS.length > 0
    ? BLOSSOM_POSITIONS
    : [new THREE.Vector3(0, 3.5, 0)];

  // ランダムな花房位置からリスポーン
  const origin = origins[Math.floor(Math.random() * origins.length)];

  petal.position.set(
    origin.x + (Math.random() - 0.5) * 0.6,
    origin.y + (Math.random() - 0.5) * 0.3,
    origin.z + (Math.random() - 0.5) * 0.6
  );

  petal.velocity.set(
    (Math.random() - 0.5) * 0.02,
    -Math.random() * 0.005, // 最初から少しだけ下向き
    (Math.random() - 0.5) * 0.02
  );

  petal.rotation.set(
    Math.random() * Math.PI * 2,
    Math.random() * Math.PI * 2,
    Math.random() * Math.PI * 2
  );

  petal.rotationSpeed.set(
    (Math.random() - 0.5) * 0.04,
    (Math.random() - 0.5) * 0.02,
    (Math.random() - 0.5) * 0.04
  );

  petal.windPhase = Math.random() * Math.PI * 2;
  petal.active = true;
}
```

---

## 🛠 修正6：Tweakpaneでリアルタイム調整UIを追加【優先度：🟢 低】

### 目的
花びら枚数・風の強さ・落下速度などをスライダーでリアルタイム調整できるパネルを追加し、「wasmにしたら枚数をここまで増やせる」をインタラクティブに証明できるようにする。

### CDN追加

```html
<!-- headタグ内に追加 -->
<script src="https://cdn.jsdelivr.net/npm/tweakpane@4.0.1/dist/tweakpane.min.js"></script>
```

### initUI() 関数を新規追加し、initScene() の最後で呼び出す

```javascript
// パラメータをオブジェクトにまとめる（PARAMS として管理）
const PARAMS = {
  petalCount:    500,
  windStrength:  0.003,
  gravity:       0.0008,
  airResistance: 0.98,
  autoRotate:    true,
};

function initUI() {
  const pane = new Tweakpane.Pane({ title: 'Sakura Controls' });

  pane.addBinding(PARAMS, 'petalCount', {
    min: 100, max: 5000, step: 100, label: 'Petals'
  }).on('change', () => {
    // 花びら数変更時はシーンをリセット
    scene.remove(petalMesh);
    initPetals(PARAMS.petalCount);
  });

  pane.addBinding(PARAMS, 'windStrength', {
    min: 0, max: 0.02, step: 0.001, label: 'Wind'
  });

  pane.addBinding(PARAMS, 'gravity', {
    min: 0.0001, max: 0.003, step: 0.0001, label: 'Gravity'
  });

  pane.addBinding(PARAMS, 'airResistance', {
    min: 0.90, max: 0.999, step: 0.001, label: 'Air Resist'
  });

  pane.addBinding(PARAMS, 'autoRotate', { label: 'Auto Rotate' })
    .on('change', (ev) => {
      controls.autoRotate = ev.value;
    });
}
```

> 💡 物理演算の `updatePetals()` 内で `GRAVITY`・`WIND_STRENGTH` などの定数を `PARAMS.gravity`・`PARAMS.windStrength` に置き換えること。

---

## 📋 修正の適用順序

AIエージェントは以下の順番で修正を適用すること：

| 順番 | 修正内容 | 理由 |
|---|---|---|
| 1 | 修正4（背景色） | 最も簡単・一発で見た目が改善 |
| 2 | 修正2（地面） | 形の土台を正しくする |
| 3 | 修正1（木・proctree.js） | `BLOSSOM_POSITIONS` を生成するため修正5より先 |
| 4 | 修正5（リスポーン位置） | 修正1で `BLOSSOM_POSITIONS` が揃ってから |
| 5 | 修正3（花びらシェーダー） | 見た目の最終仕上げ |
| 6 | 修正6（Tweakpane UI） | 最後に追加（なくても動作する） |

---

## ✅ 修正後の確認チェックリスト

| 確認項目 | 期待値 |
|---|---|
| 背景 | 単色スカイブルー `#87CEEB`、グラデーションなし |
| 地面の形 | こんもりした八角形の島、側面が黒く見える |
| 地面の色 | 明るいグリーン `#7EC850` |
| 木の形 | 幹から自然に枝分かれしている |
| 花房の位置 | 枝の先端にくっついている（宙に浮いていない） |
| 花びらの形 | 四角ではなく花びら型に見える |
| 花びらの発生元 | 花房の位置からリスポーンしている |
| fps | 500枚で60fps以上をキープ |
| Tweakpane | 右上にコントロールパネルが表示される（修正6適用時） |

---

## 🔮 修正後のさらなる改善アイデア（任意）

修正6まで完了した後に余力があれば検討：

- **花びらの色バリエーション**：1枚ごとにわずかに色相をずらす（`#F4A0C0`〜`#E8709A` の範囲でランダム）
- **着地時のフェードアウト**：地面に着いた花びらを `opacity` でゆっくり消す
- **風のアニメーション**：sin波1本でなく、複数の周波数を重ねてより自然な風に
- **カメラのイージング**：`OrbitControls` の `enableDamping: true` で慣性を追加
- **岩の改善**：`IcosahedronGeometry(detail:0)` をランダムに変形して、より自然な石に

---

*以上 — v1からv2への差分修正指示書*
