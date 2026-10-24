import {build} from 'esbuild';
import {resolve} from 'node:path';
import {mkdir} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
export async function loadSource() {
  await mkdir('.validation',{recursive:true});
  const outfile=resolve('.validation/source.cjs');
  await build({entryPoints:['tests/support/source-entry.ts'],outfile,bundle:true,platform:'node',format:'cjs',jsx:'automatic',plugins:[{name:'source-runtime-only',setup(b){
    b.onResolve({filter:/^react(?:\/jsx-runtime)?$/},()=>({path:resolve('tests/support/source-react.ts')}));
    b.onResolve({filter:/^@raycast\/(api|utils)$/},()=>({path:resolve('tests/support/source-api.ts')}));
    b.onResolve({filter:/^fuse.js$/},()=>({path:resolve('tests/fixtures/fuse-7.5.0/dist/fuse.cjs')}));
  }}]});
  return (await import(pathToFileURL(outfile))).default;
}
export function expand(element) {
  if (Array.isArray(element)) return element.flatMap(e=>{const n=expand(e);return Array.isArray(n)?n:n?[n]:[];});
  if (!element || typeof element!=='object') return element;
  if (typeof element.type==='function') return expand(element.type(element.props));
  return {...element,props:{...element.props,children:expand(element.props?.children),actions:expand(element.props?.actions),detail:expand(element.props?.detail),metadata:expand(element.props?.metadata),searchBarAccessory:expand(element.props?.searchBarAccessory)}};
}
export function walk(node,type) {
  if (!node) return [];
  if (Array.isArray(node)) return node.flatMap(n=>walk(n,type));
  return [...(node.type===type?[node]:[]),...walk(node.props?.children,type)];
}
