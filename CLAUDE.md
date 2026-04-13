# Claude Code ガイド

## ビルド・動作確認

ネイティブ（デスクトップ）と WASM（ブラウザ）の両方のビルドをサポートしています。

| 用途 | コマンド |
|------|----------|
| コンパイル確認のみ | `cargo check` |
| ネイティブで動作確認 | `cargo run` |
| ブラウザで動作確認 | `trunk serve` |
| WASM リリースビルド | `trunk build --release` |

- コンパイルエラーの確認は `cargo check` が最速です
- UI や描画の動作確認にはどちらのビルドも使用できます
- WASM 固有の問題（`web-sys`、LocalStorage など）はブラウザ確認が必要です
