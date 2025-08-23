# 設定項目

OPENAI_API_KEY = "{OPENAIのSecret Keyをここに入力してください}"
MODEL_NAME = "gpt-4o-mini"

# サーバー構成
HOST = "0.0.0.0"
PORT = 8003

# 利用回数制限
MAX_FILE_SIZE_BYTES = 20 * 1024 * 1024  # 20 MB
RATE_LIMIT_MAX_REQUESTS = 20            # IPアドレスあたりの最大リクエスト数
RATE_LIMIT_WINDOW_SECONDS = 60 * 60     # リセットの周期

# LLM設定
LLM_MAX_CANDIDATES = 800
