# GPT-4o mini File Search API

ファイルリスト(.json)とプロンプトから候補となるファイルを検索できる簡易APIです。

## Endpoint

- `POST /search-file` (multipart/form-data形式)
  - 項目:
    - `json_file`: ファイル一覧の情報を含むjson形式のファイル(`{ "files": [...] }`)。
    - `prompt`: 自然言語による検索のプロンプト(例えば「最近開いたエクセルのデータってどこ？」など)。
  - 応答: `text/plain`: 候補となるパスのリストを返します。

## Build

「かいふくまほう！」では既定で https://internal-searchfile-sendpoint.end2end.tech を利用していますが、
以下の手順にしたがって独自のサーバーにセットアップできます。

運用にはOpenAIのAPIキー(Secret Key)が必要ですので、まずは[こちら](https://platform.openai.com/api-keys)から取得してください。

また、動作にはDockerが必要です。

### 1. APIキーの設定

まずはリポジトリを複製して、取得したAPIキーを `app/config.py` の `OPENAI_API_KEY` の項目に設定してください。

### 2. ビルド

次に、以下のコマンドでDockerイメージの作成を行ってください。

```Bash
docker build -t gpt4o-mini-file-search .
```

### 3. 実行

生成したDockerイメージから、以下のコマンドでコンテナを開始してください。

```Bash
docker run --rm -p 8003:8003 gpt4o-mini-file-search
```

### 4. テスト

既定で8003番ポートにエンドポイントが用意されますので、以下のようなcurlコマンドで動作検証が可能です。

```bash
curl -X POST http://localhost:8003/search-file   -F "prompt=最近開いたエクセルのデータってどこ？"   -F "json_file=@sample.json"
```

## JSONデータの例

ファイル一覧の情報は、以下のようなjson形式で用意してください。

```
{
  "files": [
    {
      "name": "売上_四半期_Q2.xlsx",
      "path": "C:/Users/hogehoge/Documents/finance/売上_四半期_Q2.xlsx",
      "ext": ".xlsx",
      "last_opened": "2025-08-20T09:12:00",
      "last_modified": "2025-08-18T21:03:00",
      "app_hint": "Excel"
    },
    {
      "name": "memo.txt",
      "path": "C:/Users/hogehoge/Documents/memo.txt",
      "ext": ".txt",
      "last_modified": "2025-06-01T10:00:00"
    }
  ]
}
```
