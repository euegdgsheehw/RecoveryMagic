import json
from typing import Any, Dict, List, Tuple
import requests
from .config import OPENAI_API_KEY, MODEL_NAME, LLM_MAX_CANDIDATES

# 優先して検索する拡張子
EXCEL_EXTS = {".xlsx", ".xls", ".xlsm", ".xlsb", ".xltx", ".xltm", ".csv"}

# LLMに送るプロンプト
SYSTEM_PROMPT = (
    "You are a precise file selector.\n"
    "Given a user query and a set of candidate files (each with name, path, and possible metadata),\n"
    "choose the SINGLE best-matching file for the user's intent.\n"
    "Prioritize: (1) recency fields such as last_opened/last_modified if available,\n"
    "(2) exact app/type match, and (3) name/path keyword match.\n"
    "If no clear single best, return the best 3, one per line, most likely first.\n"
    "Output ONLY the file path(s) (no extra words)."
)

OPENAI_CHAT_COMPLETIONS_URL = "https://api.openai.com/v1/chat/completions"

# データをリスト化する関数
def _normalize_items(items: Any) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    if not isinstance(items, list):
        return out
    for x in items:
        if isinstance(x, str):
            out.append({"name": x, "path": x})
        elif isinstance(x, dict):
            d = {
                "name": x.get("name") or x.get("file_name") or x.get("path") or "",
                "path": x.get("path") or x.get("fullpath") or x.get("name") or "",
                "ext": x.get("ext") or x.get("extension"),
                "last_opened": x.get("last_opened") or x.get("last_accessed"),
                "last_modified": x.get("last_modified"),
                "app_hint": x.get("app_hint"),
            }
            out.append(d)
    return out

# 優先検索のフィルタリング(とりあえずExcelを優先)
def _excel_prefilter(items: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    def is_excel(d: Dict[str, Any]) -> bool:
        ext = (d.get("ext") or "").lower()
        name = (d.get("name") or d.get("path") or "").lower()
        return any(name.endswith(e) for e in EXCEL_EXTS) or (ext in EXCEL_EXTS)
    excels = [d for d in items if is_excel(d)]
    return excels if excels else items

# 候補ファイルの絞り込み
# それぞれのファイルにスコアを与えて、上限(limit)以降を切り捨てる
def _truncate_for_llm(items: List[Dict[str, Any]], limit: int) -> List[Dict[str, Any]]:
    if len(items) <= limit:
        return items
    def score(d: Dict[str, Any]) -> Tuple[int, int]:
        has_recency = 1 if (d.get("last_opened") or d.get("last_modified")) else 0
        has_path = 1 if d.get("path") else 0
        return (has_recency, has_path)
    sorted_items = sorted(items, key=score, reverse=True)
    return sorted_items[:limit]

# LLM呼び出し
def select_file(prompt: str, raw_items: Any) -> str:
    items = _normalize_items(raw_items)
    items = _excel_prefilter(items)
    items = _truncate_for_llm(items, LLM_MAX_CANDIDATES)

    lines = []
    for i, d in enumerate(items):
        line = (
            f"{i}|name={d.get('name','')}|path={d.get('path','')}"
            f"|ext={d.get('ext','')}|last_opened={d.get('last_opened','')}"
            f"|last_modified={d.get('last_modified','')}|app_hint={d.get('app_hint','')}"
        )
        lines.append(line)
    catalog = "\n".join(lines)

    user_message = (
        "User query: " + prompt.strip() + "\n\n"
        + "Candidates (index|name|path|ext|last_opened|last_modified|app_hint):\n"
        + catalog + "\n\n"
        + "Return ONLY the winning file path(s). If multiple, one per line."
    )

    headers = {
        # 認証情報はヘッダーのBearerで送る
        "Authorization": f"Bearer {OPENAI_API_KEY}",
        "Content-Type": "application/json",
    }
    payload = {
        "model": MODEL_NAME,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_message},
        ],
        "temperature": 0.0,
        "max_tokens": 200,
    }

    resp = requests.post(
        OPENAI_CHAT_COMPLETIONS_URL,
        headers=headers,
        data=json.dumps(payload),
        timeout=60,
    )
    if resp.status_code != 200:
        raise RuntimeError(f"OpenAI API error {resp.status_code}: {resp.text}")

    data = resp.json()
    try:
        content = data["choices"][0]["message"]["content"]
    except Exception as e:
        raise RuntimeError(f"Unexpected OpenAI response schema: {e}; body={data!r}")
    return (content or "").strip()
