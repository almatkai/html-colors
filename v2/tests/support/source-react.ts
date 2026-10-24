let state: unknown[] = [];
let cursor = 0;
export function resetState(values: unknown[]) { state = values; cursor = 0; }
export function useState(initial: unknown) { const i=cursor++; return [i<state.length?state[i]:initial, () => {}]; }
export function useMemo<T>(fn:()=>T) { return fn(); }
export function useCallback<T>(fn:T) { return fn; }
export const Fragment = 'Fragment';
export function jsx(type: unknown, props: unknown, key?: string) { return {type,props,key}; }
export const jsxs = jsx;
