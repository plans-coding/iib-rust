/* tslint:disable */
/* eslint-disable */

export function page_load(): void;

export function slugify(s: string): string;

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly page_load: () => void;
  readonly start: () => void;
  readonly slugify: (a: number, b: number) => [number, number];
  readonly rust_sqlite_wasm_abort: () => void;
  readonly rust_sqlite_wasm_assert_fail: (a: number, b: number, c: number, d: number) => void;
  readonly rust_sqlite_wasm_calloc: (a: number, b: number) => number;
  readonly rust_sqlite_wasm_free: (a: number) => void;
  readonly rust_sqlite_wasm_getentropy: (a: number, b: number) => number;
  readonly rust_sqlite_wasm_localtime: (a: number) => number;
  readonly rust_sqlite_wasm_malloc: (a: number) => number;
  readonly rust_sqlite_wasm_realloc: (a: number, b: number) => number;
  readonly sqlite3_os_end: () => number;
  readonly sqlite3_os_init: () => number;
  readonly wasm_bindgen__convert__closures_____invoke__h287c14aff19c8166: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hbeb953b7a7c54932: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hb6303756b209a30d: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h5901f74a6643a8c7: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
