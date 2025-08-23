const SEARCH_HOST = "https://internal-searchfile-sendpoint.end2end.tech";

function stripAnsi(s) {
  s = s.replace(/\x1B\[[0-?]*[ -/]*[@-~]/g, "");
  s = s.replace(/\x1B[@-Z\\-_]/g, "");
  s = s.replace(/\x1B\][^\x07\x1B]*(?:\x07|\x1B\\)/g, "");
  return s;
}
function stripControlExceptNL(s) {
  return s.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, "");
}
function sanitizeLog(x) {
  let s = typeof x === "string" ? x : String(x);
  s = stripAnsi(s);
  s = stripControlExceptNL(s);
  return s;
}

const ui = {
  askSection: document.getElementById("askSection"),
  askInput: document.getElementById("askInput"),
  askBtn: document.getElementById("askBtn"),
  askResults: document.getElementById("askResults"),
  select: document.getElementById("driveSelect"),
  refreshBtn: document.getElementById("refreshBtn"),
  mountBtn: document.getElementById("mountBtn"),
  ejectBtn: document.getElementById("ejectBtn"),
  progSection: document.getElementById("progressSection"),
  progBar: document.getElementById("progBar"),
  progText: document.getElementById("progText"),
  logArea: document.getElementById("logArea"),
  clearLogBtn: document.getElementById("clearLogBtn"),
  hint: document.getElementById("driveHint"),
};

function appendLog(line) {
  const at = new Date().toISOString().replace("T", " ").replace("Z", "");
  ui.logArea.value += `[${at}] ${sanitizeLog(line)}\n`;
  ui.logArea.scrollTop = ui.logArea.scrollHeight;
}

function onTauriReady(cb) {
  if (window.__TAURI_READY__) cb();
  else (window.__onTauriReadyQueue = window.__onTauriReadyQueue || []).push(cb);
}
function tauriInvoke() {
  const g = window.__TAURI__;
  const inv = g?.invoke ?? g?.tauri?.invoke;
  if (!inv) throw new Error("Tauri invoke API not found");
  return inv;
}
function tauriListen() {
  const g = window.__TAURI__;
  const listen = g?.event?.listen;
  if (!listen) throw new Error("Tauri event API not found");
  return listen;
}

let softTimer = null;
let softActive = false;
let lastPct = 0;
let realProgressSeen = false;

function setProgress(pct, msg = "") {
  lastPct = Math.max(0, Math.min(100, pct));
  ui.progBar.style.width = `${lastPct}%`;
  ui.progText.textContent = `${lastPct.toFixed(1)}%` + (msg ? ` | ${sanitizeLog(msg)}` : "");
}
function startSoftProgress() {
  stopSoftProgress();
  softActive = true;
  if (lastPct < 1.5) setProgress(1.5, "スキャン開始");
  const startAt = performance.now();
  softTimer = setInterval(() => {
    if (!softActive || realProgressSeen) return;
    const t = (performance.now() - startAt) / 1000;
    const target = 92;
    const step = Math.max(0.15, 2.2 / (1.0 + t));
    const next = Math.min(target, lastPct + step);
    setProgress(next, "スキャンしています...");
  }, 200);
}
function stopSoftProgress() {
  softActive = false;
  if (softTimer) clearInterval(softTimer), (softTimer = null);
}

function fmtIsoLocalFromEpochSec(ts) {
  try {
    const d = new Date(ts * 1000);
    const iso = new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString();
    return iso.slice(0, 19);
  } catch {
    return undefined;
  }
}
function extractPathsFromText(text) {
  const re = /[A-Za-z]:[\\/][^\s"']+/g;
  const set = new Set();
  for (const m of text.matchAll(re)) {
    const p = m[0].replace(/[),.]+$/, "");
    set.add(p);
  }
  return Array.from(set);
}

function renderAskResults(prompt, paths, rawText) {
  ui.askResults.innerHTML = "";

  const head = document.createElement("div");
  head.className = "rounded-lg border border-slate-800 bg-slate-900/60 p-3";
  head.innerHTML = `
    <div class="text-xs text-slate-400 mb-1">ファイル検索</div>
    <div class="text-sm text-slate-200 break-words">${sanitizeLog(prompt)}</div>
  `;
  ui.askResults.appendChild(head);

  if (paths.length === 0) {
    const card = document.createElement("div");
    card.className = "rounded-lg border border-slate-800 bg-slate-900/60 p-3";
    card.innerHTML = `
      <div class="text-sm text-slate-300">該当パスが見つかりませんでした。</div>
      <pre class="mt-2 text-xs text-slate-400 whitespace-pre-wrap">${sanitizeLog(rawText || "")}</pre>
    `;
    ui.askResults.appendChild(card);
    return;
  }

  for (const path of paths) {
    const card = document.createElement("div");
    card.className = "rounded-lg border border-slate-800 bg-slate-900/60 p-3";
    card.innerHTML = `
      <div class="text-xs text-slate-400 mb-1">候補パス</div>
      <div class="text-sm text-slate-200 break-all">${sanitizeLog(path)}</div>
      <div class="mt-3 flex gap-2 flex-wrap">
        <button class="px-3 py-1.5 rounded bg-sky-600 hover:bg-sky-500" data-act="open">開く</button>
        <button class="px-3 py-1.5 rounded bg-amber-600 hover:bg-amber-500" data-act="copy">デスクトップにコピー</button>
        <button class="px-3 py-1.5 rounded bg-slate-700 hover:bg-slate-600" data-act="reveal">エクスプローラーで表示</button>
      </div>
    `;
    const openBtn = card.querySelector('[data-act="open"]');
    const copyBtn = card.querySelector('[data-act="copy"]');
    const revealBtn = card.querySelector('[data-act="reveal"]');
    openBtn.addEventListener("click", () => openPath(path));
    copyBtn.addEventListener("click", () => copyToDesktop(path));
    revealBtn.addEventListener("click", () => revealInExplorer(path));
    ui.askResults.appendChild(card);
  }
}

async function askGpt() {
  if (ui.askSection.classList.contains("hidden")) {
    appendLog("gpt-4o-mini: マウント完了後に利用可能です。");
    return;
  }

  const prompt = (ui.askInput.value || "").trim();
  if (!prompt) {
    appendLog("gpt-4o-mini: プロンプトが空です。");
    return;
  }
  ui.askInput.value = "";
  ui.askInput.blur();

  appendLog(`gpt-4o-mini < "${prompt}"`);

  try {
    const invoke = tauriInvoke();
    const items = await invoke("build_filelist_cmd", { limit: 50000 });

    const files = items.map((it) => {
      const obj = { name: it.name, path: it.path, ext: it.ext || "" };
      if (typeof it.last_opened_ts === "number") obj.last_opened = fmtIsoLocalFromEpochSec(it.last_opened_ts);
      if (typeof it.last_modified_ts === "number") obj.last_modified = fmtIsoLocalFromEpochSec(it.last_modified_ts);
      return obj;
    });
    const jsonStr = JSON.stringify({ files });

    const form = new FormData();
    form.append("prompt", prompt);
    form.append("json_file", new Blob([jsonStr], { type: "application/json" }), "filelist.json");

    const url = `${SEARCH_HOST}/search-file`;
    const resp = await fetch(url, { method: "POST", body: form });
    const text = await resp.text();

    appendLog(`gpt-4o-mini > ${text}`);
    const paths = extractPathsFromText(text);
    renderAskResults(prompt, paths, text);
  } catch (e) {
    appendLog(`gpt-4o-mini: ${String(e)}`);
  }
}

async function openPath(path) {
  try {
    const invoke = tauriInvoke();
    await invoke("open_path_cmd", { path });
  } catch (e) {
    appendLog(`開くエラー: ${String(e)}`);
  }
}
async function copyToDesktop(path) {
  try {
    const invoke = tauriInvoke();
    const dest = await invoke("copy_to_desktop_cmd", { path });
    appendLog(`コピー完了: ${dest}`);
  } catch (e) {
    appendLog(`コピーに失敗しました: ${String(e)}`);
  }
}
async function revealInExplorer(path) {
  try {
    const invoke = tauriInvoke();
    await invoke("reveal_in_explorer_cmd", { path });
  } catch (e) {
    appendLog(`エクスプローラー表示に失敗しました: ${String(e)}`);
  }
}

async function loadDrives() {
  ui.select.innerHTML = `<option value="" disabled selected>読み込み中...</option>`;
  ui.mountBtn.disabled = true;

  try {
    const invoke = tauriInvoke();
    const drives = await invoke("list_drives_cmd");
    ui.select.innerHTML = "";
    if (!drives || drives.length === 0) {
      ui.select.innerHTML = `<option value="" disabled selected>候補が見つかりません</option>`;
      appendLog("ドライブの候補が見つかりませんでした。");
      return;
    }
    for (const d of drives) {
      const opt = document.createElement("option");
      opt.value = d.letter;
      opt.textContent = `${d.letter}: (空き: ${d.free} / 合計: ${d.total})`;
      ui.select.appendChild(opt);
    }
    ui.mountBtn.disabled = false;
    appendLog(`ドライブ列挙: ${drives.length} 件`);
  } catch (e) {
    ui.select.innerHTML = `<option value = "" disabled selected > エラー: ${sanitizeLog(String(e))}</option>`;
    appendLog(`ドライブ列挙エラー: ${String(e)} `);
  }
}

async function mountSelected() {
  const letter = ui.select.value;
  if (!letter) { appendLog("ドライブ未選択"); return; }
  ui.mountBtn.disabled = true;
  ui.refreshBtn.disabled = true;
  realProgressSeen = false;
  startSoftProgress();
  ui.progSection.classList.remove("hidden");
  appendLog(`マウント開始: ${letter}: `);
  try {
    const invoke = tauriInvoke();
    await invoke("start_mount_cmd", { letter });
  } catch (e) {
    appendLog(`start_mount_cmd エラー: ${String(e)} `);
    stopSoftProgress();
    ui.mountBtn.disabled = false;
    ui.refreshBtn.disabled = false;
  }
}

async function eject() {
  try {
    const invoke = tauriInvoke();
    await invoke("eject_cmd");
  } catch (e) {
    appendLog(`eject_cmd エラー: ${String(e)} `);
  }
}

async function wireEvents() {
  const listen = tauriListen();

  await listen("progress", (ev) => {
    const p = ev?.payload || {};
    if (typeof p.percent === "number") {
      realProgressSeen = true;
      setProgress(p.percent, p.msg ?? "");
    } else if (typeof p.msg === "string") {
      ui.progText.textContent = `${lastPct.toFixed(1)}% | ${sanitizeLog(p.msg)} `;
    }
  });

  await listen("state", (ev) => {
    const st = ev?.payload?.state;
    if (st === "mounted") {
      stopSoftProgress();
      setProgress(100, "マウント完了");
      ui.ejectBtn.classList.remove("hidden");
      ui.mountBtn.classList.add("hidden");
      ui.progSection.classList.add("hidden");
      ui.askSection.classList.remove("hidden");
      ui.askBtn.disabled = false;
      ui.askInput.disabled = false;
      appendLog(`マウント完了: R: \\`);
    } else if (st === "ejected") {
      stopSoftProgress();
      setProgress(0, "");
      ui.ejectBtn.classList.add("hidden");
      ui.mountBtn.classList.remove("hidden");
      ui.progText.textContent = "待機中";
      ui.progSection.classList.remove("hidden");
      ui.askSection.classList.add("hidden");
      ui.askBtn.disabled = true;
      ui.askInput.disabled = true;
      ui.askInput.value = "";
      ui.askResults.innerHTML = "";
      appendLog(`取り出し完了: R: \\ をアンマウントしました`);
      loadDrives();
      ui.refreshBtn.disabled = false;
    } else if (st === "error") {
      stopSoftProgress();
      appendLog(`エラー: ${sanitizeLog(ev?.payload?.error ?? "unknown")} `);
      ui.mountBtn.disabled = false;
      ui.refreshBtn.disabled = false;
      ui.askSection.classList.add("hidden");
      ui.askBtn.disabled = true;
      ui.askInput.disabled = true;
    }
  });

  await listen("log", (ev) => {
    const payload = ev?.payload;
    if (typeof payload === "string") appendLog(payload);
    else if (payload && typeof payload.line === "string") appendLog(payload.line);
  });
}

let __APP_STARTED__ = false;
async function startApp() {
  if (__APP_STARTED__) return;
  __APP_STARTED__ = true;

  ui.askSection.classList.add("hidden");
  ui.askBtn.disabled = true;
  ui.askInput.disabled = true;

  ui.refreshBtn.addEventListener("click", loadDrives);
  ui.mountBtn.addEventListener("click", mountSelected);
  ui.ejectBtn.addEventListener("click", eject);
  ui.clearLogBtn.addEventListener("click", () => (ui.logArea.value = ""));
  ui.askBtn.addEventListener("click", askGpt);
  ui.askInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") askGpt();
  });

  try { await wireEvents(); }
  catch (e) { appendLog(`イベントエラー: ${String(e)} `); }

  await loadDrives();
}

onTauriReady(() => {
  appendLog("Tauriの初期化が完了しました。");
  startApp().catch(e => appendLog(`起動エラー: ${String(e)} `));
});

document.addEventListener("DOMContentLoaded", () => {
  if (typeof window.__drainEarlyLogs === "function") window.__drainEarlyLogs();
});


(function pollForTauri(maxMs = 3000) {
  const start = Date.now();
  const timer = setInterval(() => {
    const g = window.__TAURI__;
    if (g?.invoke || g?.tauri?.invoke) {
      clearInterval(timer);
      if (!__APP_STARTED__) {
        startApp().catch(e => appendLog(`起動エラー: ${String(e)} `));
      }
      return;
    }
    if (Date.now() - start > maxMs) {
      clearInterval(timer);
      if (!__APP_STARTED__) {
        appendLog("Tauriを検出できませんでした。");
        startApp().catch(e => appendLog(`起動エラー: ${String(e)} `));
      }
    }
  }, 50);
})();
