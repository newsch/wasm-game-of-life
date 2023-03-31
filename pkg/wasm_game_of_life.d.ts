/* tslint:disable */
/**
*/
export function wasm_main(): void;
export enum Cell {
  Dead,
  Alive,
}
/**
*/
export enum EdgeBehavior {
  Wrap,
  Dead,
  Alive,
}
/**
*/
/**
*/
export class Universe {
  free(): void;
/**
* @param {number} width 
* @param {number} height 
* @returns {Universe} 
*/
  static new(width: number, height: number): Universe;
/**
* @param {Uint8Array} f 
*/
  reset_from_file(f: Uint8Array): void;
/**
*/
  reset_blank(): void;
/**
*/
  reset_fancy(): void;
/**
*/
  reset_random(): void;
/**
* Returns a Unicode grid in a string, representing the Universe.
* @returns {string} 
*/
  render(): string;
/**
* Updates the Universe, bringing cells into and out of existence.
*/
  tick(): void;
/**
* Updates the Universe, bringing cells into and out of existence.
*/
  tick_delta(): void;
/**
* Returns a pointer to the cells buffer.
*
* Cells are laid out as a linear stack of rows.
* Mapping from (row, col) coordinates to an index into the linear stack
* can be done with: `idx = (row_num * width + col_num)`
* @returns {number} 
*/
  cells(): number;
/**
* @returns {number} 
*/
  cells_born(): number;
/**
* @returns {number} 
*/
  cells_died(): number;
/**
* @returns {number} 
*/
  cells_born_count(): number;
/**
* @returns {number} 
*/
  cells_died_count(): number;
/**
* @param {number} row 
* @param {number} column 
*/
  toggle_cell(row: number, column: number): void;
  edge_behavior: number;
  height: number;
  width: number;
}

/**
* If `module_or_path` is {RequestInfo}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {RequestInfo | BufferSource | WebAssembly.Module} module_or_path
*
* @returns {Promise<any>}
*/
export default function init (module_or_path?: RequestInfo | BufferSource | WebAssembly.Module): Promise<any>;
        