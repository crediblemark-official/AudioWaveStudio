import { resetScreenEffectsState } from './screenEffects';
import { resetThreeDState } from './threeD';
import { resetApi3DState } from './api3D';
import { resetNeonCity3DState } from './neonCity3D';
import { resetSpiralGalaxyState } from './spiralGalaxy';
import { resetSpeakerTrioState } from './speakerTrio';
import { resetSpeakerSplatterState } from './speakerSplatter';
import { resetFlameFireState } from './flameFire';
import { resetNoiseState } from './background/noise';
import { resetTextFadeState } from './textOverlay';

export function resetVisualizerState() {
  resetTextFadeState();
  resetScreenEffectsState();
  resetThreeDState();
  resetApi3DState();
  resetNeonCity3DState();
  resetSpiralGalaxyState();
  resetSpeakerTrioState();
  resetSpeakerSplatterState();
  resetFlameFireState();
  resetNoiseState();
}
