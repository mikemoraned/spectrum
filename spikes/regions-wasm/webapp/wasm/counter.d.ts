/* tslint:disable */
/* eslint-disable */
/**
*/
export class Counter {
  free(): void;
/**
*/
  constructor();
/**
* @param {number} value
*/
  set_count(value: number): void;
/**
* @returns {number}
*/
  get_count(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_counter_free: (a: number) => void;
  readonly counter_new: () => number;
  readonly counter_set_count: (a: number, b: number) => void;
  readonly counter_get_count: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
