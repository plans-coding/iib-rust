// SQLITE -----------------------------

async function idbGet(dbName, storeName, key) {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(dbName);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => {
      const db = req.result;
      const tx = db.transaction(storeName, "readonly");
      const store = tx.objectStore(storeName);
      const getReq = store.get(key);
      getReq.onsuccess = () => resolve(getReq.result || null);
      getReq.onerror = () => reject(getReq.error);
    };
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(storeName)) {
        db.createObjectStore(storeName);
      }
    };
  });
}

async function idbPut(dbName, storeName, key, value) {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(dbName);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => {
      const db = req.result;
      const tx = db.transaction(storeName, "readwrite");
      const store = tx.objectStore(storeName);
      const putReq = store.put(value, key);
      putReq.onsuccess = () => resolve();
      putReq.onerror = () => reject(putReq.error);
    };
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(storeName)) {
        db.createObjectStore(storeName);
      }
    };
  });
}

function buffersAreEqual(buf1, buf2) {
  if (!buf1 || !buf2 || buf1.byteLength !== buf2.byteLength) return false;
  const a1 = new Uint8Array(buf1);
  const a2 = new Uint8Array(buf2);
  for (let i = 0; i < a1.length; i++) if (a1[i] !== a2[i]) return false;
  return true;
}

async function readHandle(handle) {
  const file = await handle.getFile();
  return await file.arrayBuffer();
}

async function writeHandle(handle, buffer) {
  const writable = await handle.createWritable();
  await writable.write(buffer);
  await writable.close();
}

async function readOPFSFile(path) {
  const root = await navigator.storage.getDirectory();
  try {
    const fh = await root.getFileHandle(path);
    const file = await fh.getFile();
    return await file.arrayBuffer();
  } catch (e) {
    console.log(e);
    if (e.name === "NotFoundError") return null;    
    throw e;
  }
}

async function writeOPFSFile(path, buffer) {
  const root = await navigator.storage.getDirectory();
  const fh = await root.getFileHandle(path, { create: true });
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

// ---------------------------
// Main binding function
// ---------------------------
async function bindSQLiteFile(opfsPath = "chronik.sqlite") {
  const DB_NAME = "sqlite-file-binding";
  const STORE_NAME = "handles";
  const STORE_KEY = "sqlite-db";

  let handle = await idbGet(DB_NAME, STORE_NAME, STORE_KEY);
  const hasUserGesture = !!(navigator.userActivation && navigator.userActivation.isActive);

  // If no handle persisted, prompt user
  if (!handle) {
    if (!window.showOpenFilePicker) {
      console.log("File System Access API not supported. Using OPFS only.");
      const opfsBuffer = await readOPFSFile(opfsPath);
      return { buffer: opfsBuffer, opfsHandle: null };
    }
    if (!hasUserGesture) {
      console.log("No persisted file handle and no user gesture. Using OPFS only.");
      const opfsBuffer = await readOPFSFile(opfsPath);
      return { buffer: opfsBuffer, opfsHandle: null };
    }

    const [fileHandle] = await window.showOpenFilePicker({
      types: [{ description: "SQLite DB", accept: { "application/x-sqlite3": [".sqlite"] } }],
    });
    if (!fileHandle) throw new Error("No file selected");

    handle = fileHandle;
    await idbPut(DB_NAME, STORE_NAME, STORE_KEY, handle);
  }

  // Check permission
  let perm = await handle.queryPermission({ mode: "read" });
  if (perm !== "granted" && hasUserGesture) {
    perm = await handle.requestPermission({ mode: "read" });
  }
  if (perm !== "granted") {
    console.warn("Permission denied. Using OPFS only.");
    const opfsBuffer = await readOPFSFile(opfsPath);
    return { buffer: opfsBuffer, opfsHandle: null };
  }

  // Read user file
  const userBytes = await readHandle(handle);

  // Read OPFS file
  const opfsBytes = await readOPFSFile(opfsPath);

  // Overwrite OPFS if different
  if (!buffersAreEqual(userBytes, opfsBytes)) {
    console.log("Updating OPFS from bound file...");
    await writeOPFSFile(opfsPath, userBytes);
    console.log("OPFS updated.");
  } else {
    console.log("OPFS file is already up-to-date.");
  }

  const root = await navigator.storage.getDirectory();
  const opfsHandle = await root.getFileHandle(opfsPath);

  return { buffer: userBytes, opfsHandle };
}
