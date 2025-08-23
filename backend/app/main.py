import json
from fastapi import FastAPI, File, Form, UploadFile, HTTPException, Request
from fastapi.responses import PlainTextResponse
from fastapi.middleware.cors import CORSMiddleware
from .config import (
    HOST, PORT,
    MAX_FILE_SIZE_BYTES,
    RATE_LIMIT_MAX_REQUESTS,
    RATE_LIMIT_WINDOW_SECONDS,
)
from .rate_limiter import SlidingWindowRateLimiter
from .llm import select_file

app = FastAPI(title="gpt4o-mini file search API", version="1.0.0")

# CORS(既定では全オリジンを許可。必要に応じて設定可能)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

limiter = SlidingWindowRateLimiter(RATE_LIMIT_MAX_REQUESTS, RATE_LIMIT_WINDOW_SECONDS)

def _client_ip(req: Request) -> str:
    # もしCloudFlareによるIPアドレス情報のヘッダーがあれば優先して使う
    # ちなみに internal-searchfile-sendpoint.end2end.tech もCloudFlareを利用
    cf_ip = req.headers.get("cf-connecting-ip")
    if cf_ip:
        return cf_ip.strip()
    tci = req.headers.get("true-client-ip")
    if tci:
        return tci.strip()
    xff = req.headers.get("x-forwarded-for")
    if xff:
        return xff.split(",")[0].strip()
    client = req.client
    return client.host if client else "unknown"

@app.get("/health", response_class=PlainTextResponse)
async def health() -> str:
    return "ok"

@app.post("/search-file", response_class=PlainTextResponse)
async def search_file(
    request: Request,
    json_file: UploadFile = File(..., description="JSON file containing file entries"),
    prompt: str = Form(..., description="Natural language query, e.g., '最近開いたエクセルのデータってどこ？'"),
):
    ip = _client_ip(request)
    if not limiter.allow(ip):
        raise HTTPException(status_code=429, detail="Rate limit exceeded: 20 requests per hour per IP.")

    raw = await json_file.read()
    if len(raw) > MAX_FILE_SIZE_BYTES:
        raise HTTPException(status_code=413, detail="File too large (max 20MB).")

    try:
        payload = json.loads(raw.decode("utf-8"))
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid JSON: {e}")

    items = payload.get("files") if isinstance(payload, dict) and "files" in payload else payload
    if not isinstance(items, list):
        raise HTTPException(status_code=400, detail="JSON must be a list of file entries or an object with key 'files'.")

    try:
        result_text = select_file(prompt, items)
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"LLM selection failed: {e}")

    return PlainTextResponse(content=(result_text or ""), media_type="text/plain; charset=utf-8")

# HTTPのエンドポイント立ち上げ
def run() -> None:
    import uvicorn
    uvicorn.run("app.main:app", host=HOST, port=PORT, reload=False, access_log=True)

if __name__ == "__main__":
    run()
