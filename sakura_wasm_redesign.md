# Sakura Wasm Redesign

最終更新: 2026-04-03

## 0. 実装フェーズ進捗

| Phase | Status | Scope | Done | Remaining |
|---|---|---|---|---|
| Phase 0 | completed | 見た目と単一メッシュ版の基礎実装 | 桜の木、島、岩、有限状態花びら、UI、検証フック | なし |
| Phase 1 | in_progress | JS のまま 3 レイヤー分離 | Scene / tree / ground / rocks / UI、単一メッシュ版の花びら挙動まで存在 | `ON_TREE` の GPU 化、`ON_GROUND` の静的化、`FALLING` 専用レイヤー分離、フレームループ再編 |
| Phase 2 | not_started | `FALLING` の Wasm 化 | 設計書と API 案だけ整理済み | Rust ソース、SoA バッファ、bridge、イベント返却、ビルド導線 |
| Phase 3 | not_started | partial upload と更新範囲最適化 | 方針のみ定義 | `activeIndices` ベースの転送最適化、`setMatrixAt()` 排除 |
| Phase 4 | planned | 10 万枚目標の追加最適化 | 将来方針のみ | GPU 主体化や WebGPU 寄せを評価 |

### 現在地

- 現在の `index.html` は **Phase 0 完了 / Phase 1 Step 1-4 相当まで完了** の状態
- Phase 1 Step 5-8 にあたる 3 レイヤー構成、shader sway、静的 ground layer、フレームループ再編は未着手
- まだ単一 `InstancedMesh` に依存しているため、現時点の公開 UI 上限は安全側の `24000`

## 1. 前提

現状の `index.html` は `InstancedMesh` を使っているが、花びら更新はすべて JavaScript で行っている。
そのため、`wasm` の利点である「大量の数値計算を連続メモリ上で高速に回す」点を活かせていない。

特に重いのは次の 4 点:

1. 全花びらを毎フレーム CPU で総当たりしている
2. 毎フレーム `setMatrixAt()` を大量に呼んでいる
3. 毎フレーム巨大な instance 行列バッファを GPU に再送している
4. `ON_TREE` / `FALLING` / `ON_GROUND` を同じ更新経路で扱っている

この構成では、仮に計算部分だけを `wasm` 化しても、ボトルネックのかなりの部分が残る。

## 2. 再設計のゴール

### 目標

- 10 万枚以上でも極端に破綻しない構造にする
- `wasm` は「落下中の花びらシミュレーション」に集中投入する
- `ON_TREE` と `ON_GROUND` はなるべく CPU 更新対象から外す
- 毎フレームの GPU 転送量を最小化する
- JavaScript は UI と scene orchestration に限定する

### 非目標

- 全状態を wasm で持つこと自体を目的にしない
- とりあえず全部 wasm に移す、という設計は採用しない
- three.js を捨ててフルスクラッチの WebGL にすることは初期段階では行わない

## 3. 新しい責務分離

### JavaScript が担うもの

- three.js scene の初期化
- カメラ、ライト、UI、入力
- Wasm メモリの確保と bridge
- `ON_TREE` / `FALLING` / `ON_GROUND` 各描画レイヤーの更新タイミング制御
- `render_game_to_text()` とデバッグ表示

### Wasm が担うもの

- `FALLING` 花びらの位置・速度・回転の更新
- `fallDelay` 到達判定
- 着地判定
- `activeIndices` 管理
- `ON_GROUND` への遷移イベント生成

### GPU / シェーダーが担うもの

- `ON_TREE` 花びらの微揺れ
- 花びら形状のカットアウト
- `ON_TREE` / `ON_GROUND` の最終頂点変換
- 可能なら `FALLING` も行列ではなく軽量 attribute から変換

## 4. 状態の分離設計

現状は全花びらを 1 配列で持っているが、再設計後は状態ごとに更新方法を変える。

### 4-1. ON_TREE

特徴:

- 初期状態では大半がここに属する
- 基本的には静的
- 「揺れ」は見た目の演出であり、物理計算ではない

扱い:

- CPU / wasm では毎フレーム更新しない
- `basePosition`, `baseRotation`, `phase`, `scale` だけを保持
- 頂点シェーダーで `time` と `phase` から揺れを作る
- `fallDelay` 到達時にだけ `FALLING` へ移す

重要:

- `ON_TREE` を毎フレーム `setMatrixAt()` しない
- これだけで CPU コストと GPU 転送量が大幅に下がる

### 4-2. FALLING

特徴:

- 実際に毎フレーム更新が必要なのはここだけ
- `wasm` を最も活かせる区間

扱い:

- `activeCount` と `activeIndices` を持つ
- `wasm` は active な花びらだけを SoA で更新
- 着地したら active リストから外す

重要:

- 全件ループ禁止
- `FALLING` のみ更新対象にする

### 4-3. ON_GROUND

特徴:

- 着地後は静的
- 数は最終的に最も増える

扱い:

- 着地時に 1 回だけ GPU 用バッファへ書き込む
- 以後は更新しない
- `ON_GROUND` レイヤーは静的インスタンス群として持つ

重要:

- `ON_GROUND` を毎フレーム CPU 更新しない

## 5. 描画レイヤー構成

描画は 1 メッシュにこだわらず、状態別に分ける方が合理的。

### Layer A: Tree Petals Mesh

- 対象: `ON_TREE`
- 実装: `InstancedMesh` または `InstancedBufferGeometry`
- 更新: 基本なし
- 揺れ: 頂点シェーダー
- 毎フレーム更新するのは `uniform time` 程度

### Layer B: Falling Petals Mesh

- 対象: `FALLING`
- 実装: `InstancedBufferGeometry`
- attribute: `position`, `rotation`, `scale`, `brightness`
- 更新: wasm 計算結果を partial upload

### Layer C: Ground Petals Mesh

- 対象: `ON_GROUND`
- 実装: `InstancedBufferGeometry`
- 更新: 着地時だけスロットに書き込む
- 毎フレーム更新なし

この 3 層構造にすることで、
「動いているものだけ頻繁に更新する」設計にできる。

## 6. データレイアウト

`wasm` に向いた構造にするため、AoS ではなく SoA を採用する。

### 悪い例

```js
petals.push({
  position: { x, y, z },
  velocity: { x, y, z },
  rotation: { x, y, z },
  ...
});
```

これはオブジェクト参照が多く、`wasm` や高速ループに不向き。

### 採用する構造

```text
posX[petalCount]
posY[petalCount]
posZ[petalCount]

velX[petalCount]
velY[petalCount]
velZ[petalCount]

rotX[petalCount]
rotY[petalCount]
rotZ[petalCount]

rotSpeedX[petalCount]
rotSpeedY[petalCount]
rotSpeedZ[petalCount]

phase[petalCount]
scale[petalCount]
fallDelay[petalCount]
state[petalCount]
```

さらに `FALLING` 用には:

```text
activeIndices[activeCount]
```

を持つ。

## 7. Wasm 化するべき場所

### 優先度 A: 落下シミュレーション本体

これは最優先で wasm 化するべき。

対象:

- `fallDelay` 到達判定
- 重力
- 風の合成
- 空気抵抗
- 位置更新
- 回転更新
- 着地判定
- active list の compaction

理由:

- 大量の単純計算
- 分岐は比較的単純
- SoA と相性が良い
- JavaScript より wasm の効果が出やすい

### 優先度 B: falling -> ground の遷移イベント生成

着地した index をイベントバッファに積んで JS 側へ返す。

対象:

- landedIndices の書き出し
- activeCount の更新

理由:

- JS で全件を再走査しなくて済む
- ground layer への書き込み対象を限定できる

### 優先度 C: 初期化時のランダム生成の一部

これは後回し。

対象候補:

- `fallDelay`
- `phase`
- `rotationSpeed`

ただし初期化は毎フレームではないため、優先度は低い。

## 8. Wasm 化しない方がよい場所

### scene 構築

- three.js の scene graph
- camera / controls / lighting
- DOM UI

これは JS のままでよい。

### ON_TREE の揺れ

これは wasm ではなく GPU シェーダーへ移すべき。

理由:

- 毎フレーム CPU で更新するのが無駄
- `time + phase` だけで表現できる
- GPU が最も得意

### 花びら形状マスク

既にシェーダーでやっているので、そのままでよい。

## 9. ブリッジ API 案

### JS -> Wasm

```text
initPetalBuffers(count)
seedPetals(seed)
activateReadyPetals(frameCount)
stepFalling(dt, time, windStrength, gravity, airResistance)
getActiveCount()
getLandedCount()
getLandedIndicesPtr()
getFallingBufferPtr()
```

### Wasm -> JS

JS はポインタ経由で typed array view を貼る。

```js
const mem = wasm.instance.exports.memory.buffer;
const posX = new Float32Array(mem, ptrPosX, maxPetals);
```

### JS 側の責務

- landed index を受け取って ground layer へ 1 回だけ書く
- active falling instance buffer を GPU へ反映
- tree layer は `time` uniform だけ更新

## 10. フレーム処理の新ループ

理想的な 1 フレームはこうなる。

1. JS が `time` と `dt` を計算
2. JS が wasm に `activateReadyPetals(frameCount)` を呼ぶ
3. JS が wasm に `stepFalling(...)` を呼ぶ
4. JS が landed events を取得
5. JS が ground mesh に landed 分だけ追記
6. JS が falling mesh の active 範囲だけ GPU に更新
7. JS が tree shader に `time` を渡す
8. render

重要:

- `ON_TREE` 全件更新なし
- `ON_GROUND` 全件更新なし
- `FALLING` の active 範囲のみ更新

## 11. 現行コードから見た具体的な問題点

### 問題 1: 全件ループ

現行の `updatePetals(time)` は全花びらを毎フレーム走査している。

### 問題 2: 行列更新中心

各花びらに対して `dummy.updateMatrix()` と `setMatrixAt()` を行っている。

これは数が増えると CPU 側で詰まりやすい。

### 問題 3: 状態ごとの差が薄い

`ON_TREE` と `ON_GROUND` は本来「静的寄り」なのに、現在の設計では毎フレーム処理パスに巻き込まれる。

### 問題 4: wasm を入れる場所がない

今のデータ構造はオブジェクト中心で、wasm に渡しても恩恵を出しにくい。

## 12. 推奨実装順序

### Phase 1

- 現行 JS のまま、状態別 3 レイヤーに分離
- `ON_TREE` を shader sway 化
- `ON_GROUND` を静的化
- `FALLING` のみ CPU 更新

この段階でかなり軽くなる。

### Phase 2

- `FALLING` シミュレーションを wasm 化
- SoA へ移行
- `activeIndices` 管理を導入

### Phase 3

- `FALLING` の GPU 転送を partial update 化
- 必要なら `InstancedBufferGeometry` に寄せる

### Phase 4

- 本当に数十万を狙うなら、falling も GPU 主体に寄せる
- wasm は event / scheduling / spawn 管理へ寄せる

## 13. 結論

### いまの結論

- 現状は wasm を活かせていない
- 理由は「wasm がない」だけではなく、設計が数十万向けではないため

### 何を wasm 化するべきか

最優先:

- `FALLING` の数値更新ループ
- active list 管理
- 着地イベント生成

wasm 化しない方がいいもの:

- scene / UI / three.js 制御
- `ON_TREE` の揺れ
- 花びら形状シェーダー

### 本当に効く設計

- `ON_TREE` は GPU
- `FALLING` は wasm
- `ON_GROUND` は静的
- JS は orchestration のみ

この構成なら、初めて wasm の利点を正しく使える。
