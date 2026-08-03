import { readFileSync } from 'node:fs';
const IN = '/tmp/awcmp-stress/inputs';
for (const f of ['000', '009', '015', '029']) {
  const bin = readFileSync(`${IN}/frame_${f}.bin`);
  const freq = new Uint8Array(bin.buffer, bin.byteOffset, 512);
  const b16 = Array.from(freq.slice(0, 16));
  const sum16 = b16.reduce((a, b) => a + b, 0);
  const first8 = Array.from(freq.slice(0, 8));
  const sum8 = first8.reduce((a, b) => a + b, 0);
  console.log(`frame_${f}: bass bins[0..15]=${b16.join(',')}`);
  console.log(`  sum16=${sum16} (${(sum16/(16*255)).toFixed(3)})  sum8=${sum8} (${(sum8/(8*255)).toFixed(3)})  bins[16..31]=${Array.from(freq.slice(16,32)).join(',')}`);
}
