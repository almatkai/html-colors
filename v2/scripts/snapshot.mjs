// Generate/verify the offline Rust regression corpus using original source code.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {loadSource} from './source-loader.mjs';
import {createRequire} from 'node:module';
const source=await loadSource();const require=createRequire(import.meta.url);
const Fuse=require('../tests/fixtures/fuse-7.5.0/dist/fuse.cjs');
const all=[...source.basicColors,...source.extendedColors];
const queries=['',' ','\t','Red','r','ff','#FF0000','rgb(255, 0, 0)','rgb(255,0,0)','CornflwerBlue','DarkSlateGre','LightBlue','blu','Red ',' Red','İNDIGO','KBlue','😀blue','😀'.repeat(15)+'Red','rgb(255, 0, 0)'.repeat(3),'red'.repeat(22)];
const cases=[];for(const view of ['all','basic','extended','grouped']){
 const input=view==='basic'?source.basicColors:view==='extended'?source.extendedColors:all;
 const fuse=new Fuse(input,{keys:[{name:'name',weight:2},{name:'hex',weight:1},{name:'rgb',weight:1}],threshold:0.5,distance:150,includeScore:true,shouldSort:true,minMatchCharLength:2});
 for(const query of queries){const rows=source.searchColors(input,query);cases.push({view,query,rows,sections:source.sortSectionsByRelevance(Object.entries(source.groupColorsByShade(rows)),query),ranked:fuse.search(query).filter(r=>r.score!==undefined&&r.score<0.5).map(r=>[all.indexOf(r.item),r.score])});}
}
const data={generatedBy:'scripts/snapshot.mjs executing own TS source and locked Fuse 7.5.0',sourceCommit:'ccd3bcb56fa43e993381dec63d3b3e7e9403bac0',colors:all,cases};
const path='tests/fixtures/oracle.json';
if(process.argv.includes('--write'))writeFileSync(path,JSON.stringify(data)+'\n');else assert.deepEqual(JSON.parse(readFileSync(path,'utf8')),data);
console.log(`source-derived offline corpus: ${cases.length} view/query cases`);
