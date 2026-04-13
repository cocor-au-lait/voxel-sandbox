# Voxel Sandbox

ブラウザで動く Minecraft 風 3D サンドボックスゲーム。
Rust + [Bevy](https://bevyengine.org/) で実装し、WebAssembly にコンパイルして GitHub Pages で公開しています。

> **このプロジェクトは全コードを [Claude Code](https://claude.ai/code) のみで実装しています。**
> 人間はゲームの仕様を指示し、Claude Code がコーディング・デバッグ・コミットをすべて行いました。

## プレイ

**[https://cocor-au-lait.github.io/voxel-sandbox/](https://cocor-au-lait.github.io/voxel-sandbox/)**

## 操作方法

| 入力 | 操作 |
|------|------|
| クリック | カーソルロック / ゲーム開始 |
| WASD | 移動 |
| Space | ジャンプ |
| マウス移動 | 視点操作 |
| 左クリック | ブロック破壊 |
| 右クリック | ブロック設置 |
| 1 〜 9 | ブロック選択 |
| スクロール | ブロック選択 |
| ESC | カーソル解放 |

## 実装状況

- [x] チャンクベースのワールド管理（32³ボクセル）
- [x] チャンク動的ロード / アンロード（描画距離 6 チャンク）
- [x] プロシージャル地形生成（Fbm\<Perlin\> ノイズ、高低差 ±30 ブロック）
- [x] バイオーム（草原・砂漠・森）、洞窟、木の生成
- [x] 可視面カリングメッシュ生成（フレーム分割処理付き）
- [x] テクスチャアトラス（terrain.png、256×256、16×16 タイル）
- [x] FPS カメラ（pitch/yaw、視野角 70°）
- [x] WASD 移動・ジャンプ・重力
- [x] AABB 衝突判定
- [x] DDA レイキャストによるブロック選択（到達距離 6 ブロック）
- [x] ブロック破壊・設置
- [x] ホットバー UI（9スロット、テクスチャ表示）
- [x] スクロール / 数字キーによるブロック選択
- [x] ブロック選択ハイライト
- [x] クロスヘア
- [ ] セーブ / ロード
- [ ] 操作説明オーバーレイ（テキスト描画の問題で一時削除）
- [ ] Greedy Meshing による描画最適化
- [ ] クラフト・インベントリ画面（E キー）
- [ ] タイトル・メインメニュー画面
- [ ] ローディング画面
- [ ] 昼夜サイクル・空のアニメーション

## アーキテクチャ

Bevy の ECS（Entity Component System）をベースに、Plugin 単位でシステムを分割しています。

```
src/
├── main.rs              # アプリエントリポイント、ワールド初期設定
├── block/               # BlockType enum、ブロック属性（衝突判定・テクスチャ座標・色）
├── chunk/               # ChunkData（32³ボクセル）、ChunkDataStore、動的ロード/アンロード
├── terrain/             # チャンク座標からボクセルデータを生成する地形ジェネレータ
├── meshing/             # 隣接チャンク参照による可視面カリング、MeshingQueue
├── rendering/           # テクスチャアトラス（terrain.png）、全チャンク共有マテリアル
├── player/
│   ├── camera.rs        # FPS カメラ（pitch/yaw）、カーソルロック管理
│   ├── movement.rs      # WASD 移動、重力、ジャンプ
│   ├── collision.rs     # AABB 衝突解決
│   └── interaction.rs   # DDA レイキャスト、ブロック設置/破壊、HUD
├── inventory/           # ホットバー（9スロット）、SelectedBlock リソース同期
└── persistence/         # ワールドのシリアライズ/デシリアライズ、ストレージ抽象化
```

### 技術スタック

| | |
|---|---|
| 言語 | Rust（2024 edition） |
| ゲームエンジン | Bevy 0.18 |
| 描画ターゲット | WebAssembly + WebGL2 |
| 地形ノイズ | `noise` クレート（Fbm\<Perlin\>） |
| シリアライズ | `serde` + `serde_json` |
| Web ストレージ | `web-sys`（LocalStorage） |

### 設計上のポイント

- **チャンクサイズ 32³** — WASM シングルスレッド環境でメッシュ生成が 1〜3ms に収まるサイズ
- **テクスチャアトラス** — 256×256 PNG を `include_bytes!` でバイナリに埋め込み、WASM でのアセット読み込み問題を回避
- **Tonemapping::None** — `TonyMcMapface` は `tonemapping_luts` feature（zstd 依存）が必要で WASM 非対応のため無効化
- **dt クランプ（0.05秒）** — バックグラウンドタブから復帰時に大きな dt で物理演算が暴走するのを防止

## ローカル確認

### 事前準備（初回のみ）

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

### ブラウザで確認（trunk serve）

```bash
trunk serve
```

`http://localhost:8080` でブラウザ確認できます。ファイル変更を監視して自動リビルドします。

### ネイティブで確認

```bash
cargo run
```

## ビルド

```bash
# WASM リリースビルド（GitHub Actions で自動デプロイ）
trunk build --release
```
