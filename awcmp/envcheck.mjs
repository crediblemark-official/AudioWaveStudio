// Replicate the TS canvasRenderer envelope (export path) to see pulse values.
import { readFileSync } from 'node:fs';

const IN = '/tmp/awcmp-stress/inputs';
const FPS = 30;
let bassEnergy = 0, bassEnergyRaw = 0, beatStrength = 0, beatStrengthRaw = 0;
let prevTargetBass = 0, prevRawBass = 0, bassFloor = 0;
const bassMult = 1.6, sens = 1.5, pulseIntensity = 0.7;

for (let f = 0; f < 30; f++) {
  const bin = readFileSync(`${IN}/frame_${String(f).padStart(3, '0')}.bin`);
  const freq = new Uint8Array(bin.buffer, bin.byteOffset, 512);
  let bassSum = 0;
  const bassBins = Math.min(16, freq.length);
  for (let i = 0; i < bassBins; i++) bassSum += freq[i];
  const rawBass = bassSum / (bassBins * 255);
  const targetBass = rawBass * bassMult * sens;
  bassEnergy += (targetBass - bassEnergy) * 0.2;
  bassEnergyRaw += (rawBass - bassEnergyRaw) * 0.2;
  if (targetBass < bassFloor) bassFloor = targetBass;
  else bassFloor += (targetBass - bassFloor) * 0.0008;
  const aboveFloor = Math.max(0, bassEnergy - bassFloor);
  const onset = Math.max(0, targetBass - prevTargetBass);
  prevTargetBass = targetBass;
  if (onset > 0.03) beatStrength = Math.max(onset * 6, beatStrength * 0.6);
  else beatStrength *= 0.7;
  const rawOnset = Math.max(0, rawBass - (prevRawBass || 0));
  prevRawBass = rawBass;
  beatStrengthRaw = rawOnset > 0.06 ? Math.max(rawOnset * 5, beatStrengthRaw * 0.5) : beatStrengthRaw * 0.5;
  if (f === 0 || f === 9 || f === 15 || f === 29) {
    const beat = beatStrength > 0.15 ? beatStrength * pulseIntensity : 0;
    const smooth = bassEnergy * pulseIntensity * 0.15;
    const alpha = smooth + beat;
    console.log(`frame ${String(f).padStart(2)}: rawBass=${rawBass.toFixed(3)} target=${targetBass.toFixed(3)} bassEnergy=${bassEnergy.toFixed(3)} aboveFloor=${aboveFloor.toFixed(3)} beatStrength=${beatStrength.toFixed(3)} pulseAlpha=${alpha.toFixed(3)}`);
  }
}
