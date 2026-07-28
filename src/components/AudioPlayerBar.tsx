import React, { useEffect, useState } from 'react';
import { Play, Pause, Square, Volume2, VolumeX, Music } from 'lucide-react';
import { SongMetadata } from '../types/visualizer';
import { audioEngine } from '../services/audioEngine';

interface AudioPlayerBarProps {
  songMeta: SongMetadata | null;
}

export const AudioPlayerBar: React.FC<AudioPlayerBarProps> = ({ songMeta }) => {
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [currentTime, setCurrentTime] = useState<number>(0);
  const [duration, setDuration] = useState<number>(0);
  const [volume, setVolumeState] = useState<number>(0.8);
  const [isMuted, setIsMuted] = useState<boolean>(false);

  useEffect(() => {
    audioEngine.setTimeUpdateCallback((time) => {
      setCurrentTime(time);
      setIsPlaying(audioEngine.getIsPlaying());
    });

    audioEngine.setEndedCallback(() => {
      setIsPlaying(false);
      setCurrentTime(0);
    });
  }, []);

  useEffect(() => {
    if (songMeta) {
      setDuration(songMeta.duration);
      setIsPlaying(audioEngine.getIsPlaying());
    } else {
      setDuration(0);
      setCurrentTime(0);
    }
  }, [songMeta]);

  const handlePlayPause = async () => {
    console.log('[PlayerBar] handlePlayPause', { songMeta: !!songMeta, isPlaying });
    if (!songMeta) return;
    if (isPlaying) {
      audioEngine.pause();
      setIsPlaying(false);
    } else {
      await audioEngine.play();
      setIsPlaying(audioEngine.getIsPlaying());
    }
  };

  const handleStop = () => {
    audioEngine.stop();
    setIsPlaying(false);
    setCurrentTime(0);
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = parseFloat(e.target.value);
    audioEngine.seek(val);
    setCurrentTime(val);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = parseFloat(e.target.value);
    setVolumeState(val);
    audioEngine.setVolume(val);
    if (val > 0 && isMuted) setIsMuted(false);
  };

  const toggleMute = () => {
    if (isMuted) {
      audioEngine.setVolume(volume);
      setIsMuted(false);
    } else {
      audioEngine.setVolume(0);
      setIsMuted(true);
    }
  };

  const formatTime = (secs: number) => {
    if (isNaN(secs) || secs <= 0) return '00:00';
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };

  return (
    <div className="audio-player-bar">
      {/* Song Info */}
      <div className="player-track-info">
        <Music size={20} className="text-secondary" />
        <div className="track-details">
          <span className="track-title">{songMeta?.title || 'No Song Loaded'}</span>
          <span className="track-artist">{songMeta?.artist || 'Import an audio file to start'}</span>
        </div>
      </div>

      {/* Controls & Seeker */}
      <div className="player-center">
        <div className="player-controls">
          <button
            className={`btn-play ${isPlaying ? 'playing' : ''}`}
            onClick={handlePlayPause}
            disabled={!songMeta}
            title={isPlaying ? 'Pause' : 'Play'}
          >
            {isPlaying ? <Pause size={20} /> : <Play size={20} className="ml-1" />}
          </button>
          <button
            className="btn-control"
            onClick={handleStop}
            disabled={!songMeta}
            title="Stop"
          >
            <Square size={16} />
          </button>
        </div>

        <div className="player-seeker">
          <span className="time-text">{formatTime(currentTime)}</span>
          <div className="range-wrapper">
            <input
              type="range"
              min={0}
              max={duration || 100}
              step={0.1}
              value={currentTime}
              onChange={handleSeek}
              disabled={!songMeta}
              className="seek-slider"
            />
            <div
              className="seek-progress"
              style={{ width: `${duration ? (currentTime / duration) * 100 : 0}%` }}
            />
          </div>
          <span className="time-text">{formatTime(duration)}</span>
        </div>
      </div>

      {/* Volume Control */}
      <div className="player-right">
        <button className="btn-icon" onClick={toggleMute} title="Mute/Unmute">
          {isMuted || volume === 0 ? <VolumeX size={18} /> : <Volume2 size={18} />}
        </button>
        <input
          type="range"
          min={0}
          max={1}
          step={0.02}
          value={isMuted ? 0 : volume}
          onChange={handleVolumeChange}
          className="volume-slider"
        />
      </div>
    </div>
  );
};
