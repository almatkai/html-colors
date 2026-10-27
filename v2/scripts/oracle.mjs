import assert from 'node:assert/strict';
import {spawn,execFileSync} from 'node:child_process';
import {createInterface} from 'node:readline';
import {createRequire} from 'node:module';
import {loadSource,expand,walk} from './source-loader.mjs';
const source=await loadSource();
const require=createRequire(import.meta.url);
const Fuse=require('../tests/fixtures/fuse-7.5.0/dist/fuse.cjs');
assert.equal(Fuse.version,'7.5.0');
execFileSync('cargo',['build','--locked','--release','--example','oracle_bridge'],{stdio:'inherit'});
const rust=spawn('target/release/examples/oracle_bridge',[],{stdio:['pipe','pipe','inherit']});
const lines=createInterface({input:rust.stdout})[Symbol.asyncIterator]();
async function bridge(value) {rust.stdin.write(JSON.stringify(value)+'\n');const line=await lines.next();assert.ok(!line.done,'Rust exited early');return JSON.parse(line.value);}
const all=[...source.basicColors,...source.extendedColors];
const dataset=await bridge({colors:true});assert.deepEqual(dataset.colors,all);
const options={keys:[{name:'name',weight:2},{name:'hex',weight:1},{name:'rgb',weight:1}],threshold:0.5,distance:150,includeScore:true,shouldSort:true,minMatchCharLength:2};
const queries=new Set(['',' ','  ','\t','\n',' Red','Red ',' red ','#','r','00','ff','rgb','rgb(255, 0, 0)','rgb(255,0,0)','bule','gren','rebeccapurple','blüe','İ','Σ','K','İNDIGO','😀red','red😀','\u00a0','\u2003','\u0000','𝔅lue','𐐀','RED','000000','rgb(0, 0, 0)']);
for (const c of all) for (const text of [c.name,c.hex,c.rgb,c.id]) {
  queries.add(text);queries.add(text.toLowerCase());queries.add(' '+text);queries.add(text+' ');
  queries.add(text.slice(0,Math.max(2,text.length-1)));queries.add(text.slice(1));
  queries.add(text.slice(0,2)+'x'+text.slice(3));queries.add(text.slice(0,1)+text.slice(2));
}
for(let i=0;i<256;i++) {
  const text=all[(i*37)%all.length].name;
  queries.add(text.slice(0,i%text.length)+'😀'+text.slice(i%text.length));
  queries.add(('red #FF0000 rgb(255, 0, 0) '+text+' ').repeat(1+i%4));
}
for (const n of [31,32,33,34,63,64,65,95,96,97,128,150,256,1024]) {
  for (const text of ['Red','rgb(255, 0, 0)','😀Red','KBlue',' ']) {
    queries.add(text.repeat(Math.ceil(n/text.length)).slice(0,n));
  }
}
const rejectedUnpairedSurrogates=[...queries].filter(q=>!q.isWellFormed()).length;
const corpus=[...queries].filter(q=>q.isWellFormed());let comparisons=0,scoreChecks=0;
for (const view of ['all','basic','extended','grouped']) {
  const input=view==='basic'?source.basicColors:view==='extended'?source.extendedColors:all;
  const fuse=new Fuse(input,options);
  for (const [queryIndex,query] of corpus.entries()) {
    if(queryIndex%1000===0)console.log(`oracle ${view} ${queryIndex}/${corpus.length}`);
    const actual=await bridge({view,query});
    const expected=source.searchColors(input,query);
    assert.deepEqual(actual.rows,expected,`${view} rows ${JSON.stringify(query)}`);
    const sections=source.sortSectionsByRelevance(Object.entries(source.groupColorsByShade(expected)),query);
    assert.deepEqual(actual.sections,sections,`${view} sections ${JSON.stringify(query)}`);
    const ranked=fuse.search(query).filter(r=>r.score!==undefined&&r.score<0.5).sort((a,b)=>(a.score||0)-(b.score||0));
    assert.deepEqual(actual.ranked.map(r=>r[0]),ranked.map(r=>all.indexOf(r.item)),`${view} rank ${JSON.stringify(query)}`);
    actual.ranked.forEach((r,i)=>{assert.ok(Math.abs(r[1]-ranked[i].score)<2e-14,`score ${query}: ${r[1]} vs ${ranked[i].score}`);scoreChecks++;});
    comparisons++;
  }
}
// Execute the original TSX components and original copy handlers for every row.
let copyChecks=0,tsxViews=0;
for (const view of ['all','basic','extended','grouped']) for (const query of ['',' ','Red','#ff0000','rgb(255, 0, 0)','bule','red'.repeat(22),'😀blue']) for(const showHex of [true,false]) for(const details of [false,true]) {
  source.resetState([query,view,showHex,details]);
  const tree=expand(source.Command());
  const rows=walk(tree,'List.Item');
  const actual=await bridge({view,query,wire:true,details,showHex});
  const nativeRows=[];function nativeWalk(n){if(n.type==='List.Item')nativeRows.push(n);else for(const c of n.children??[])nativeWalk(c);}nativeWalk(actual.wire.root);
  assert.equal(tree.props.searchBarPlaceholder,actual.wire.root.props.searchBarPlaceholder);
  assert.equal(rows.length,nativeRows.length);
  assert.deepEqual(rows.map(r=>r.props.title),nativeRows.map(r=>r.props.title));
  for(let i=0;i<rows.length;i++){
    const a=rows[i].props,b=nativeRows[i].props;
    const sourceActions=walk(a.actions,'Action');const nativeActions=b.actionIds.map(id=>actual.wire.actions.find(a=>a.id===id));
    assert.deepEqual(sourceActions.map(a=>a.props.title),nativeActions.map(a=>a.title));
    assert.ok(nativeActions.every(a=>a.kind==='action'));
    for(const j of [3,4])assert.deepEqual(sourceActions[j].props.shortcut,nativeActions[j].shortcut);
    assert.equal(a.subtitle,b.subtitle);assert.deepEqual(a.icon,b.icon);assert.deepEqual(a.accessories,b.accessories);
    if(details){assert.equal(a.detail.props.markdown,b.detail.props.markdown);assert.deepEqual(walk(a.detail.props.metadata,'List.Item.Detail.Metadata.Label').map(n=>n.props).map(({children,actions,detail,metadata,searchBarAccessory,...p})=>p),b.detail.props.metadata.children.filter(c=>c.type.endsWith('.Label')).map(c=>c.props));}
  }
  const sourceSections=walk(tree,'List.Section').map(n=>n.props.title);
  assert.deepEqual(sourceSections,(actual.wire.root.children??[]).filter(n=>n.type==='List.Section').map(n=>n.props.title));
  for (const row of rows) {
    const actions=walk(row.props.actions,'Action');
    assert.deepEqual(actions.map(a=>a.props.title),['Copy HEX to Clipboard','Copy RGB to Clipboard','Copy Name to Clipboard',showHex?'Show RGB':'Show HEX',details?'Hide Details':'Show Details']);
    for (const [i,field] of ['hex','rgb','id'].entries()) {
      await actions[i].props.onAction();
      const color=all.find(c=>c.name===row.props.title);
      assert.equal(source.copies.at(-1),color[field]);copyChecks++;
    }
  }
  tsxViews++;
}
assert.notEqual(source.searchColors(all,'').length,source.searchColors(all,' ').length);
rust.stdin.end();await new Promise((resolve,reject)=>rust.on('exit',c=>c===0?resolve():reject(new Error(`Rust ${c}`))));
console.log(JSON.stringify({queries:corpus.length,rejectedUnpairedSurrogates,viewComparisons:comparisons,scoreChecks,tsxViews,sourceCopyHandlerChecks:copyChecks}));
