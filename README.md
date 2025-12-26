# Radio Recorder

NHK（らじる★らじる）とradikoの番組を録音・再生できるデスクトップアプリ。

## 機能

- **NHK対応（認証不要）**
  - NHKラジオ第1 / 第2 / FM
  - 番組表からワンクリック録音

- **radiko対応**
  - プレミアム会員対応（全国の局）
  - 番組表連携

- **ライブラリ管理**
  - 録音ファイルを局別に整理
  - アプリ内で再生（シークバー、速度調整）

## 技術スタック

- **Frontend**: React + TypeScript
- **Backend**: Rust (Tauri)
- **録音**: ffmpeg

## セットアップ

### 必要なもの

- Node.js 18+
- Rust
- ffmpeg

### インストール

```bash
# 依存パッケージのインストール
npm install

# 開発サーバー起動
npm run tauri dev

# ビルド
npm run tauri build
```

### ffmpegのインストール

```bash
# macOS
brew install ffmpeg

# Windows
winget install ffmpeg

# Ubuntu
sudo apt install ffmpeg
```

## フォルダ構成

録音ファイルは `~/RadioLibrary/` に保存されます：

```
~/RadioLibrary/
├── NHK-R1/
│   └── 2025-12-26_番組名.mp3
├── NHK-FM/
├── TBSラジオ/
└── metadata.json
```

## 使い方

1. アプリを起動
2. 「放送局」タブを選択
3. NHK または radiko を選択
4. 局を選んで番組表を表示
5. 「録音」ボタンをクリック

## ライセンス

MIT
