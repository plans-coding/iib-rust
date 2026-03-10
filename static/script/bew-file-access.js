// FOR BEWEGUNG ONLINE DEMO

function runIfOnBewegungOnline() {
  try {
    const url = new URL(window.location.href);
    const isTargetOrigin = url.origin === 'https://online.bewegung.app';
    const isRootPath = url.pathname === '/' || url.pathname === '';

    if (isTargetOrigin && isRootPath) {
      const btn = document.getElementById('btnFetch');
      if (!btn) {
        console.warn('Element with id "btnFetch" not found.');
        return;
      }
      if (typeof btn.click === 'function') {
        btn.click();
        return;
      }
    }
  } catch (err) {
    console.error('Invalid URL', err);
  }
}

// SQLITE -----------------------------
async function read_opfs_text(path) {
  const buffer = await readOPFSFile(path);
  if (!buffer) return null;
  return new TextDecoder().decode(buffer);
}

// NEW CODE
// ─── Pure helpers (no DOM, safe at module level) ──────────

const OPFS_DB_NAME   = 'app.sqlite';
const OPFS_META_NAME = 'app.sqlite.meta.json';
const PATH_TXT       = 'server_db_path.txt';
const LS_MODE_KEY    = 'sqlite-loader-mode';
const IDB_NAME       = 'sqlite-opfs-loader';
const IDB_STORE      = 'handles';
const IDB_KEY        = 'boundFile';

function fmtSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024, sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

async function hashBuffer(buf) {
  const digest = await crypto.subtle.digest('SHA-256', buf);
  return [...new Uint8Array(digest)].map(b => b.toString(16).padStart(2, '0')).join('');
}

// ─── OPFS helpers ─────────────────────────────────────────

async function readOPFSFile(path) {
  const root = await navigator.storage.getDirectory();
  try {
    const fh   = await root.getFileHandle(path);
    const file = await fh.getFile();
    return await file.arrayBuffer();
  } catch (e) {
    if (e.name === 'NotFoundError') return null;
    throw e;
  }
}

async function writeOPFSFile(path, buffer) {
  const root     = await navigator.storage.getDirectory();
  const fh       = await root.getFileHandle(path, { create: true });
  const writable = await fh.createWritable();
  await writable.write(buffer);
  await writable.close();
}

async function read_opfs_file(path) {
  return await readOPFSFile(path);
}

async function read_opfs_text(path) {
  const buffer = await readOPFSFile(path);
  if (!buffer) return null;
  return new TextDecoder().decode(buffer);
}

async function readOpfsMeta() {
  const raw = await readOPFSFile(OPFS_META_NAME);
  if (!raw) return null;
  try { return JSON.parse(new TextDecoder().decode(raw)); } catch { return null; }
}

async function writeOpfsMeta(meta) {
  await writeOPFSFile(OPFS_META_NAME, new TextEncoder().encode(JSON.stringify(meta)));
}

async function getOpfsDbInfo() {
  try {
    const root = await navigator.storage.getDirectory();
    const fh   = await root.getFileHandle(OPFS_DB_NAME);
    const file = await fh.getFile();
    return { size: file.size, lastModified: file.lastModified };
  } catch {
    return null;
  }
}

// ─── IndexedDB for FileSystemFileHandle persistence ───────

function idbOpen() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, 1);
    req.onupgradeneeded = () => {
      if (!req.result.objectStoreNames.contains(IDB_STORE))
        req.result.createObjectStore(IDB_STORE);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror   = () => reject(req.error);
  });
}

async function idbSaveHandle(handle) {
  const db = await idbOpen();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(IDB_STORE, 'readwrite');
    tx.objectStore(IDB_STORE).put(handle, IDB_KEY);
    tx.oncomplete = () => resolve();
    tx.onerror    = () => reject(tx.error);
  });
}

async function idbLoadHandle() {
  const db = await idbOpen();
  return new Promise((resolve, reject) => {
    const tx  = db.transaction(IDB_STORE, 'readonly');
    const req = tx.objectStore(IDB_STORE).get(IDB_KEY);
    req.onsuccess = () => resolve(req.result || null);
    req.onerror   = () => reject(req.error);
  });
}

async function idbDeleteHandle() {
  const db = await idbOpen();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(IDB_STORE, 'readwrite');
    tx.objectStore(IDB_STORE).delete(IDB_KEY);
    tx.oncomplete = () => resolve();
    tx.onerror    = () => reject(tx.error);
  });
}

// ═══════════════════════════════════════════════════════════
//  sync_db_init()  — call this once when the DOM is ready
// ═══════════════════════════════════════════════════════════

function sync_db_init() {

  const $  = (s) => document.querySelector(s);
  const $$ = (s) => document.querySelectorAll(s);

  // ─── Log ────────────────────────────────────────────────
  function log(msg, level = 'info') {
    const el  = $('#log');
    const ts  = new Date().toLocaleTimeString();
    const div = document.createElement('div');
    div.className = `log-line ${level}`;
    div.innerHTML = `<span class="ts">${ts}</span><span class="msg">${msg}</span>`;
    el.appendChild(div);
    el.scrollTop = el.scrollHeight;
  }

  $('#btnClearLog').addEventListener('click', () => { $('#log').innerHTML = ''; });

  // ─── Mode persistence & switching ───────────────────────
  const panels = {
    fetch:  $('#panel-fetch'),
    folder: $('#panel-folder'),
    upload: $('#panel-upload'),
  };

  function activateMode(mode) {
    $$('.option-card').forEach(c => {
      const match = c.dataset.mode === mode;
      c.classList.toggle('selected', match);
      c.querySelector('input[type="radio"]').checked = match;
    });
    Object.entries(panels).forEach(([k, p]) => p.classList.toggle('hidden', k !== mode));
    try { localStorage.setItem(LS_MODE_KEY, mode); } catch {}
  }

  const savedMode = (() => {
    try {
      const v = localStorage.getItem(LS_MODE_KEY);
      return (v && panels[v]) ? v : 'fetch';
    } catch { return 'fetch'; }
  })();
  activateMode(savedMode);

  $$('.option-card').forEach(card => {
    card.addEventListener('click', () => activateMode(card.dataset.mode));
  });

  // ─── Status bar ─────────────────────────────────────────
  async function refreshStatus() {
    const hasOpfs = !!navigator.storage?.getDirectory;
    $('#dotOpfs').className = `status-dot ${hasOpfs ? 'green' : 'red'}`;
    $('#valOpfs').textContent = hasOpfs ? 'Supported' : 'Not supported';
    if (!hasOpfs) return;

    const info = await getOpfsDbInfo();
    if (info) {
      $('#dotDb').className   = 'status-dot green';
      $('#valDb').textContent = OPFS_DB_NAME;
      $('#dotSize').className   = 'status-dot green';
      $('#valSize').textContent = fmtSize(info.size);
    } else {
      $('#dotDb').className     = 'status-dot';
      $('#valDb').textContent   = '—';
      $('#dotSize').className   = 'status-dot';
      $('#valSize').textContent = '—';
    }

    const handle = await idbLoadHandle();
    if (handle) {
      $('#dotBound').className   = 'status-dot yellow';
      $('#valBound').textContent = handle.name;
    } else {
      $('#dotBound').className   = 'status-dot';
      $('#valBound').textContent = '—';
    }
  }

  // ─── 1) Fetch from server ───────────────────────────────
  $('#btnFetch').addEventListener('click', async () => {
    const btn = $('#btnFetch');
    btn.disabled = true;
    const bar = $('#fetchBar');
    $('#fetchProgress').classList.remove('hidden');
    bar.style.width = '0%';

    try {
      let url = $('#fetchPathOverride').value.trim();

      if (!url) {
        log('Fetching DB path from <code>' + PATH_TXT + '</code>…');
        bar.style.width = '10%';
        const pathResp = await fetch(PATH_TXT);
        if (!pathResp.ok) throw new Error(`Could not fetch ${PATH_TXT} (${pathResp.status})`);
        url = (await pathResp.text()).trim();
        log(`Resolved path → <code>${url}</code>`);
      } else {
        log(`Using manual URL → <code>${url}</code>`);
      }

      bar.style.width = '20%';
      log('Downloading SQLite file…');

      const resp = await fetch(url);
      if (!resp.ok) throw new Error(`Download failed (${resp.status})`);

      const contentLength = +resp.headers.get('Content-Length') || 0;
      const reader  = resp.body.getReader();
      const chunks  = [];
      let received  = 0;

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        chunks.push(value);
        received += value.length;
        if (contentLength) bar.style.width = `${20 + 60 * (received / contentLength)}%`;
      }

      const blob = new Uint8Array(received);
      let offset = 0;
      for (const chunk of chunks) { blob.set(chunk, offset); offset += chunk.length; }

      log(`Downloaded ${fmtSize(received)}. Writing to OPFS…`);
      bar.style.width = '90%';
      await writeOPFSFile(OPFS_DB_NAME, blob);
      bar.style.width = '100%';
      log(`✅ Saved to OPFS as <code>${OPFS_DB_NAME}</code>`, 'ok');
      await refreshStatus();
    } catch (err) {
      log(`❌ ${err.message}`, 'error');
    } finally {
      btn.disabled = false;
      location.reload(); //Added by Plans
    }
  });

  // ─── 2) Bind local file ─────────────────────────────────
  let boundFileHandle = null;

  function showBoundUI(handle) {
    boundFileHandle = handle;
    $('#btnResync').classList.remove('hidden');
    $('#btnUnbind').classList.remove('hidden');
  }

  function hideBoundUI() {
    boundFileHandle = null;
    $('#btnResync').classList.add('hidden');
    $('#btnUnbind').classList.add('hidden');
    $('#folderInfo').classList.add('hidden');
    $('#folderProgress').classList.add('hidden');
  }

  async function syncFileToOpfs(fileHandle, force = false) {
    const bar = $('#folderBar');
    $('#folderProgress').classList.remove('hidden');
    bar.style.width = '0%';

    const file         = await fileHandle.getFile();
    const buf          = await file.arrayBuffer();
    const hash         = await hashBuffer(buf);
    const size         = file.size;
    const lastModified = file.lastModified;

    bar.style.width = '30%';

    $('#folderFilename').textContent = file.name;
    $('#folderMeta').textContent     = `${fmtSize(size)} · Modified ${new Date(lastModified).toLocaleString()}`;
    $('#folderInfo').classList.remove('hidden');

    const meta = await readOpfsMeta();
    bar.style.width = '50%';

    if (!force && meta && meta.hash === hash) {
      bar.style.width = '100%';
      log('File unchanged (SHA-256 matches). OPFS is up-to-date.', 'ok');
      return false;
    }

    if (meta && meta.hash !== hash) {
      log(`⚠ File changed! Old hash: <code>${meta.hash.slice(0, 12)}…</code> → New: <code>${hash.slice(0, 12)}…</code>`, 'warn');
    }

    log(`Writing ${fmtSize(size)} to OPFS…`);
    bar.style.width = '70%';
    await writeOPFSFile(OPFS_DB_NAME, buf);
    bar.style.width = '85%';

    await writeOpfsMeta({
      hash, size, lastModified,
      name: file.name,
      syncedAt: Date.now(),
    });

    bar.style.width = '100%';
    log(`✅ Synced <code>${file.name}</code> → OPFS`, 'ok');
    await refreshStatus();
    return true;
  }

  async function requestPermissionAndSync(handle, force = false) {
    const perm = await handle.queryPermission({ mode: 'read' });
    if (perm === 'granted') {
      await syncFileToOpfs(handle, force);
      return true;
    }

    log('Requesting read permission for <code>' + handle.name + '</code>…', 'warn');
    const req = await handle.requestPermission({ mode: 'read' });
    if (req !== 'granted') {
      log('❌ Permission denied. Cannot read the file.', 'error');
      return false;
    }

    await syncFileToOpfs(handle, force);
    return true;
  }

  $('#btnPickFile').addEventListener('click', async () => {
    try {
      if (!window.showOpenFilePicker) {
        log('❌ File System Access API not supported. Use Chrome or Edge.', 'error');
        return;
      }

      const [handle] = await window.showOpenFilePicker({
        types: [{
          description: 'SQLite databases',
          accept: { 'application/x-sqlite3': ['.sqlite', '.db', '.sqlite3'] },
        }],
        multiple: false,
      });

      log(`Picked file: <code>${handle.name}</code>`);
      await idbSaveHandle(handle);
      log('File handle saved to IndexedDB (persists across reloads).', 'ok');

      showBoundUI(handle);
      await syncFileToOpfs(handle, true);
      await refreshStatus();
    } catch (err) {
      if (err.name === 'AbortError') return;
      log(`❌ ${err.message}`, 'error');
    }
  });

  $('#btnResync').addEventListener('click', async () => {
    if (!boundFileHandle) { log('No file bound. Pick a file first.', 'warn'); return; }
    try {
      log('Re-checking file for changes…');
      await requestPermissionAndSync(boundFileHandle, false);
    } catch (err) {
      log(`❌ ${err.message}`, 'error');
    }
  });

  $('#btnUnbind').addEventListener('click', async () => {
    try {
      await idbDeleteHandle();
      hideBoundUI();
      log('File binding removed.', 'ok');
      await refreshStatus();
    } catch (err) {
      log(`❌ ${err.message}`, 'error');
    }
  });

  // ─── Auto-restore binding on page load ──────────────────
  async function restoreBinding() {
    let handle;
    try { handle = await idbLoadHandle(); } catch { return; }
    if (!handle) return;

    log(`Found persisted file handle: <code>${handle.name}</code>. Attempting auto-sync…`);
    showBoundUI(handle);

    const meta = await readOpfsMeta();
    if (meta) {
      $('#folderFilename').textContent = meta.name;
      $('#folderMeta').textContent     = `Last synced ${new Date(meta.syncedAt).toLocaleString()} · ${fmtSize(meta.size)}`;
      $('#folderInfo').classList.remove('hidden');
    }

    const perm = await handle.queryPermission({ mode: 'read' });
    if (perm === 'granted') {
      log('Permission already granted. Checking for changes…', 'ok');
      await syncFileToOpfs(handle, false);
    } else {
      log('⚠ Browser requires a user gesture to re-grant file access. Click <strong>Re-check & Sync</strong> to authorize and sync.', 'warn');
    }
  }

  // ─── 3) Upload directly ─────────────────────────────────
  const dropZone  = $('#dropZone');
  const fileInput = $('#fileInput');

  async function handleUploadFile(file) {
    if (!file) return;
    const bar = $('#uploadBar');
    $('#uploadProgress').classList.remove('hidden');
    bar.style.width = '0%';

    log(`Uploading <code>${file.name}</code> (${fmtSize(file.size)})…`);
    bar.style.width = '20%';

    try {
      const buf = await file.arrayBuffer();
      bar.style.width = '60%';

      const header = new Uint8Array(buf.slice(0, 16));
      const sig    = new TextDecoder().decode(header);
      if (!sig.startsWith('SQLite format 3')) {
        log('⚠ File does not appear to be a valid SQLite database. Saving anyway.', 'warn');
      }

      await writeOPFSFile(OPFS_DB_NAME, buf);
      bar.style.width = '100%';
      log(`✅ Uploaded <code>${file.name}</code> → OPFS as <code>${OPFS_DB_NAME}</code>`, 'ok');
      await refreshStatus();
    } catch (err) {
      log(`❌ ${err.message}`, 'error');
    }
  }

  fileInput.addEventListener('change', (e) => {
    handleUploadFile(e.target.files[0]);
    fileInput.value = '';
  });

  dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.classList.add('dragover');
  });

  dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('dragover');
  });

  dropZone.addEventListener('drop', (e) => {
    e.preventDefault();
    dropZone.classList.remove('dragover');
    handleUploadFile(e.dataTransfer.files[0]);
  });

  // ─── Boot ───────────────────────────────────────────────
  (async () => {
    log('Initializing…');

    if (!navigator.storage?.getDirectory) {
      log('❌ OPFS not supported in this browser. Use a Chromium-based browser.', 'error');
    } else {
      log('OPFS is available ✓', 'ok');
    }

    await refreshStatus();
    await restoreBinding();
  })();
}
