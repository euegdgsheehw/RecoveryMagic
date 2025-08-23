# かいふくまほう！🔮🖥️✨

削除されたファイルを「元のディレクトリ構成そのまま」で仮想ドライブとしてマウントできる、全く新しい次世代のファイル復元ツールです。

内部では NTFS の $MFT を読み取り、削除済みエントリをインデックス化して Dokan 経由で読み取り専用の仮想ファイルシステムを構築します。
R:\ にドライブとしてマウントして、通常のエクスプローラやアプリから読み取り可能にします。

また、独自機能 "ファイルをAIに探してもらう" を実装しており、「さっき消しちゃったExcelのデータない？」と自然言語で問いかけるだけで、簡単に削除したファイルを発見できます。

![Gy-KdqQboAU64kV](https://github.com/user-attachments/assets/3f1dfc23-2e13-439a-a6a2-b1e2c5477f90)

## 🛠️ 使い方

###  1. Dokanyの準備

まず初めに、[こちら](https://github.com/dokan-dev/dokany/releases/download/v2.2.1.1000/DokanSetup.exe)からDokanyをインストールしてください。

###  2. リポジトリの複製

次に、本リポジトリを複製するか、[Releases](https://github.com/ActiveTK/RecoveryMagic/archive/refs/heads/main.zip)からダウンロードして展開してください。

```bash
git clone https://github.com/ActiveTK/RecoveryMagic
cd RecoveryMagic/bin
```

### 3. かいふくまほう！の実行

1. `RecoveryMagic.exe` を起動
2. 復元したいファイルを含むドライブを指定
3. 「マウント」ボタンをクリック！
4. 🎈 仮想ドライブ `R:\` が出現！削除したデータだけを含むドライブが現れます！ 🎈

## 🧰 ソースからビルドする方法

本アプリは Tauri（Rust + GUI）を使う構成です。Rustとcargo、及びDokanをあらかじめインストールしておいてください。

リポジトリを複製した後、以下のコマンドでビルドができます。

```bash
cargo build
```

リリースとしてビルドする場合には、以下のコマンドを実行してください。

```bash
cargo build --release
```

## ⚠️ 注意事項

- 🔓 実行には管理者権限が必要です。
- 🔥 巨大なドライブを扱うとメモリ消費が増加します。
- 💾 既にデータがドライブ上から完全に消失している場合は復元できません。
- ⚠️ ファイルをAIに探してもらう機能ではファイル一覧を外部ホストへPOSTします*。

※独自にOpenAIのSecret Keyを取得して、自前でファイル検索用のサーバーを[構築](https://github.com/ActiveTK/RecoveryMagic/blob/main/backend/README.md)することもできます。

## 📦 必要ライブラリ・依存

- 📦 [Dokan.NET](https://github.com/dokan-dev/dokan-dotnet)

※ 本アプリはWindows専用ですのでご注意ください。

## 📄 ライセンス

このプログラムは The MIT License の下で公開されています。

© 2025 ActiveTK.  
🔗 https://github.com/ActiveTK/RecoveryMagic/blob/main/LICENSE
