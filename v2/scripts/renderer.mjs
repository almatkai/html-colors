import assert from 'node:assert/strict';
import {build} from 'esbuild';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {JSDOM} from 'jsdom';
import React from 'react';
import {renderToStaticMarkup} from 'react-dom/server';
import TestRenderer,{act} from 'react-test-renderer';
import {readFileSync} from 'node:fs';
const dom=new JSDOM('<!doctype html><html><body></body></html>',{url:'https://tezbar.test'});
globalThis.window=dom.window;globalThis.document=dom.window.document;globalThis.DOMParser=dom.window.DOMParser;
let osEffects=0;window.tezbar=new Proxy({}, {get(){return ()=>{osEffects++;throw Error('OS route forbidden');};}});
const require=createRequire(import.meta.url);
async function bundle(name,entry){await build({entryPoints:[entry],outfile:`.validation/${name}.cjs`,bundle:true,platform:'node',format:'cjs',jsx:'automatic',external:['react','react/jsx-runtime','react-dom']});return require(`../.validation/${name}.cjs`);}
const beta='tests/fixtures/beta/src/renderer/src/raycast-api/';
const {ListRuntime}=await bundle('beta-list',beta+'list-runtime.tsx');
const {ActionPanelOverlay}=await bundle('beta-actions',beta+'action-runtime-overlay.tsx');
const outputs=JSON.parse('['+execFileSync('target/release/examples/oracle_bridge',{input:[{view:'all',query:'CornflwerBlue',wire:true,details:true},{view:'grouped',query:'Red',wire:true,details:true},{view:'all',query:'',wire:true,details:false}].map(v=>JSON.stringify(v)).join('\n')+'\n',maxBuffer:32e6}).toString().trim().split('\n').join(',')+']');
let rowChecks=0;
for (const out of outputs) {
 const wire=out.wire;assert.ok(wire.actions.every(a=>['action','copy','open','push','pop','submit-form','show-in-finder'].includes(a.kind)));const invoked=[],opened=[],searched=[];
 const props={root:wire.root,title:wire.title,commandName:'html-color',actions:wire.actions,onBack:()=>{},onRunPrimaryAction:(id,values)=>invoked.push({id,values}),onOpenActions:ids=>opened.push(ids),onSearchTextChanged:q=>searched.push(q),onLoadMore:()=>{}};
 const html=renderToStaticMarkup(React.createElement(ListRuntime,props));
 const doc=new JSDOM(html).window.document;
 assert.ok(doc.querySelector('input[placeholder="Name, hex, or RGB..."]'));
 assert.ok(doc.querySelectorAll('[data-idx]').length>0);
 const titles=[...doc.querySelectorAll('[data-idx] p:first-child')].map(p=>p.textContent);
 const wireRows=[];function walk(n){if(n.type==='List.Item')wireRows.push(n);else for(const c of n.children??[])walk(c);}walk(wire.root);
 assert.deepEqual(titles,wireRows.map(n=>n.props.title),'server fuzzy rows survive client filtering');
 for(const [i,button] of [...doc.querySelectorAll('[data-idx]')].entries()){
  const img=button.querySelector('img');assert.equal(img?.getAttribute('src'),wireRows[i].props.icon.source);
  const svg=Buffer.from(img.getAttribute('src').split(',')[1],'base64').toString();assert.match(svg,/<circle cx="8" cy="8" r="8"/);rowChecks++;
 }
 const buttons=[...doc.querySelectorAll('button')].map(b=>b.textContent);
 for(const title of ['All Colors','Basic Colors','Extended Colors','Group by Shade'])assert.ok(buttons.includes(title));
 if(wire.root.props.isShowingDetail){
  const row=wireRows[0].props;const detail=row.detail.props;
  const preview=doc.querySelector(`img[alt="${row.title} Color Preview"]`);assert.ok(preview,'actual Markdown sanitizer retains preview data URI');
  const svg=Buffer.from(preview.src.split(',')[1],'base64').toString();assert.match(svg,/width="200" height="150"/);assert.match(svg,/rx="8" ry="8"/);
  for(const label of detail.metadata.children.filter(c=>c.type.endsWith('Label'))){assert.ok(html.includes(label.props.title));assert.ok(doc.body.textContent.includes(label.props.text));}
 }
 let renderer;await act(async()=>{renderer=TestRenderer.create(React.createElement(ListRuntime,props));});
 const root=renderer.root;
 const rows=root.findAll(n=>n.type==='button'&&n.props['data-idx']!==undefined);
 await act(async()=>rows[0].props.onClick());assert.equal(invoked.at(-1).id,wireRows[0].props.actionIds[0]);
 const extended=root.findAllByType('button').find(b=>b.children.join('')==='Extended Colors');
 await act(async()=>extended.props.onClick());assert.deepEqual(invoked.at(-1),{id:wire.root.props.searchBarAccessory.props.actionId,values:{value:'extended'}});
 const input=root.findAllByType('input')[0];
 // The pinned renderer keeps the current local draft when a same-query view refresh arrives.
 await act(async()=>input.props.onChange({target:{value:' blu '}}));
 await act(async()=>renderer.update(React.createElement(ListRuntime,{...props,root:{...wire.root,props:{...wire.root.props,isLoading:false}}})));
 assert.equal(renderer.root.findAllByType('input')[0].props.value,' blu ');
 const keyboard=root.findAll(n=>typeof n.props.onKeyDown==='function')[0];
 const event=key=>({key,metaKey:true,ctrlKey:false,altKey:false,shiftKey:false,repeat:false,preventDefault(){},stopPropagation(){}});
 let before=invoked.length;await act(async()=>{keyboard.props.onKeyDown(event('t'));keyboard.props.onKeyDown(event('i'));});assert.equal(invoked.length,before,'beta has no arbitrary shortcuts');
 await act(async()=>keyboard.props.onKeyDown(event('k')));assert.deepEqual(opened.at(-1),wireRows[0].props.actionIds);
 await act(async()=>renderer.unmount());
 const ids=wireRows[0].props.actionIds;
 const actions=ids.map(id=>wire.actions.find(a=>a.id===id));
 const menu=renderToStaticMarkup(React.createElement(ActionPanelOverlay,{actions,onClose(){},onExecute(){}}));
 const menuDoc=new JSDOM(menu).window.document;
 assert.deepEqual([...menuDoc.querySelectorAll('li button')].map(n=>n.textContent),actions.map(a=>a.title));
}
assert.equal(osEffects,0);
const source=readFileSync(beta+'list-runtime.tsx','utf8');
assert.match(source,/if \(!value\) return null/); // icon-only category accessories are not painted by beta.
console.log(JSON.stringify({betaRendererCases:outputs.length,swatchRows:rowChecks,realClipboardOrBrowserCalls:osEffects,limitations:['icon-only accessories hidden','mask ignored; circle in SVG','cmd+t/cmd+i descriptors not dispatched','selection is renderer index; dropdown resets to first row']}));
