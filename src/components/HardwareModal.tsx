import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Cpu, Zap, CheckCircle2, RefreshCw, X, Film, Monitor, Info } from 'lucide-react';

export interface GpuAdapterInfo {
  name: String;
  device_type: string;
  backend: string;
  vendor_id: number;
}

export interface EncoderCapability {
  id: string;
  name: string;
  supported: boolean;
  description: string;
}

export interface HardwareInfo {
  gpus: GpuAdapterInfo[];
  ffmpeg_installed: boolean;
  ffmpeg_path?: string;
  encoders: EncoderCapability[];
  recommended_encoder: string;
  recommended_label: string;
  os: string;
  arch: string;
}

interface HardwareModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const HardwareModal: React.FC<HardwareModalProps> = ({ isOpen, onClose }) => {
  const [loading, setLoading] = useState(false);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchHardware = async () => {
    setLoading(true);
    setError(null);
    try {
      const info = await invoke<HardwareInfo>('check_hardware');
      setHardware(info);
    } catch (err: any) {
      setError(err?.toString() || 'Gagal mendeteksi hardware.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      fetchHardware();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const isGpuAccelerated = hardware && hardware.recommended_encoder !== 'libx264';

  const getDeviceTypeLabel = (type: string) => {
    switch (type) {
      case 'IntegratedGpu':
        return 'Integrated GPU (Terintegrasi)';
      case 'DiscreteGpu':
        return 'Discrete GPU (VGA Diskrit / Dedicated)';
      case 'Cpu':
        return 'CPU Software Rendering';
      case 'VirtualGpu':
        return 'Virtual / Passthrough GPU';
      default:
        return type;
    };
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-container hardware-modal"
        onClick={(e) => e.stopPropagation()}
        style={{ maxWidth: '680px', width: '92%' }}
      >
        <div className="modal-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: 36,
                height: 36,
                borderRadius: '10px',
                background: 'rgba(59, 130, 246, 0.15)',
                color: '#3b82f6',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Cpu size={20} />
            </div>
            <div>
              <h2 style={{ margin: 0, fontSize: '1.25rem', fontWeight: 600 }}>Diagnosa Hardware & GPU</h2>
              <p style={{ margin: 0, fontSize: '0.8rem', color: '#94a3b8' }}>
                Deteksi otomatis GPU, backend grafis WGPU, dan enkoder video FFmpeg
              </p>
            </div>
          </div>
          <button className="btn-icon" onClick={onClose} aria-label="Close">
            <X size={18} />
          </button>
        </div>

        <div className="modal-body" style={{ padding: '20px 24px', maxHeight: '75vh', overflowY: 'auto' }}>
          {loading ? (
            <div style={{ padding: '40px 0', textAlign: 'center', color: '#94a3b8' }}>
              <RefreshCw className="spin" size={32} style={{ margin: '0 auto 12px auto' }} />
              <p style={{ margin: 0, fontSize: '0.95rem' }}>Mendeteksi teknologi GPU & FFmpeg...</p>
            </div>
          ) : error ? (
            <div
              style={{
                padding: '16px',
                borderRadius: '12px',
                background: 'rgba(239, 68, 68, 0.1)',
                border: '1px solid rgba(239, 68, 68, 0.2)',
                color: '#f87171',
                fontSize: '0.9rem',
              }}
            >
              {error}
            </div>
          ) : hardware ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              {/* Hero Readiness Banner */}
              <div
                style={{
                  padding: '16px 20px',
                  borderRadius: '14px',
                  background: isGpuAccelerated
                    ? 'linear-gradient(135deg, rgba(34, 197, 94, 0.12), rgba(16, 185, 129, 0.05))'
                    : 'linear-gradient(135deg, rgba(234, 179, 8, 0.12), rgba(245, 158, 11, 0.05))',
                  border: isGpuAccelerated
                    ? '1px solid rgba(34, 197, 94, 0.3)'
                    : '1px solid rgba(234, 179, 8, 0.3)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '14px',
                }}
              >
                <div
                  style={{
                    width: 44,
                    height: 44,
                    borderRadius: '12px',
                    background: isGpuAccelerated ? 'rgba(34, 197, 94, 0.2)' : 'rgba(234, 179, 8, 0.2)',
                    color: isGpuAccelerated ? '#4ade80' : '#facc15',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  {isGpuAccelerated ? <Zap size={24} /> : <Cpu size={24} />}
                </div>
                <div>
                  <div style={{ fontSize: '0.75rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.5px', color: isGpuAccelerated ? '#4ade80' : '#facc15' }}>
                    Status Akselerasi Hardware
                  </div>
                  <div style={{ fontSize: '1rem', fontWeight: 600, color: '#f8fafc', marginTop: '2px' }}>
                    {hardware.recommended_label}
                  </div>
                  <div style={{ fontSize: '0.8rem', color: '#94a3b8', marginTop: '2px' }}>
                    {isGpuAccelerated
                      ? 'Proses ekspor video MP4 memanfaatkan chip hardware GPU untuk performa rendering ultra cepat.'
                      : 'Ekspor video saat ini menggunakan Software CPU (libx264).'}
                  </div>
                </div>
              </div>

              {/* GPU Hardware Section */}
              <div
                style={{
                  background: 'rgba(30, 41, 59, 0.5)',
                  borderRadius: '14px',
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                  padding: '18px 20px',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '14px' }}>
                  <Monitor size={18} style={{ color: '#38bdf8' }} />
                  <h3 style={{ margin: 0, fontSize: '0.95rem', fontWeight: 600, color: '#f1f5f9' }}>
                    Perangkat Grafis / GPU ({hardware.gpus.length})
                  </h3>
                </div>

                {hardware.gpus.length === 0 ? (
                  <p style={{ margin: 0, fontSize: '0.85rem', color: '#94a3b8' }}>
                    Tidak ada GPU WGPU terdeteksi secara langsung.
                  </p>
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                    {hardware.gpus.map((gpu, idx) => (
                      <div
                        key={idx}
                        style={{
                          padding: '12px 14px',
                          borderRadius: '10px',
                          background: 'rgba(15, 23, 42, 0.6)',
                          border: '1px solid rgba(255, 255, 255, 0.05)',
                          display: 'flex',
                          justifyContent: 'space-between',
                          alignItems: 'center',
                          flexWrap: 'wrap',
                          gap: '8px',
                        }}
                      >
                        <div>
                          <div style={{ fontSize: '0.9rem', fontWeight: 600, color: '#f8fafc' }}>
                            {gpu.name || 'GPU Controller'}
                          </div>
                          <div style={{ fontSize: '0.78rem', color: '#94a3b8', marginTop: '2px' }}>
                            {getDeviceTypeLabel(gpu.device_type)}
                          </div>
                        </div>
                        <div style={{ display: 'flex', gap: '6px' }}>
                          <span
                            style={{
                              fontSize: '0.72rem',
                              padding: '3px 8px',
                              borderRadius: '6px',
                              background: 'rgba(56, 189, 248, 0.15)',
                              color: '#38bdf8',
                              fontWeight: 500,
                            }}
                          >
                            Backend: {gpu.backend}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* FFmpeg & Encoders Table */}
              <div
                style={{
                  background: 'rgba(30, 41, 59, 0.5)',
                  borderRadius: '14px',
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                  padding: '18px 20px',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <Film size={18} style={{ color: '#a855f7' }} />
                    <h3 style={{ margin: 0, fontSize: '0.95rem', fontWeight: 600, color: '#f1f5f9' }}>
                      Teknologi Enkoder Video FFmpeg
                    </h3>
                  </div>
                  <span
                    style={{
                      fontSize: '0.75rem',
                      padding: '3px 10px',
                      borderRadius: '12px',
                      background: hardware.ffmpeg_installed ? 'rgba(34, 197, 94, 0.15)' : 'rgba(239, 68, 68, 0.15)',
                      color: hardware.ffmpeg_installed ? '#4ade80' : '#f87171',
                      fontWeight: 600,
                    }}
                  >
                    {hardware.ffmpeg_installed ? 'FFmpeg Ready' : 'FFmpeg Belum Ada'}
                  </span>
                </div>

                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  {hardware.encoders
                    .filter((enc) => enc.supported)
                    .map((enc) => {
                      const isRec = enc.id === hardware.recommended_encoder;
                      return (
                        <div
                          key={enc.id}
                          style={{
                            padding: '10px 14px',
                            borderRadius: '10px',
                            background: isRec ? 'rgba(59, 130, 246, 0.12)' : 'rgba(15, 23, 42, 0.4)',
                            border: isRec ? '1px solid rgba(59, 130, 246, 0.3)' : '1px solid rgba(255, 255, 255, 0.04)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            gap: '10px',
                          }}
                        >
                          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                            <CheckCircle2 size={16} style={{ color: '#4ade80', flexShrink: 0 }} />
                            <div>
                              <div style={{ fontSize: '0.85rem', fontWeight: 600, color: '#f1f5f9' }}>
                                {enc.name} <code style={{ fontSize: '0.75rem', opacity: 0.7 }}>({enc.id})</code>
                                {isRec && (
                                  <span
                                    style={{
                                      marginLeft: '8px',
                                      fontSize: '0.68rem',
                                      padding: '2px 6px',
                                      borderRadius: '4px',
                                      background: '#3b82f6',
                                      color: '#ffffff',
                                      fontWeight: 600,
                                    }}
                                  >
                                    Aktif
                                  </span>
                                )}
                              </div>
                              <div style={{ fontSize: '0.75rem', color: '#94a3b8', marginTop: '1px' }}>
                                {enc.description}
                              </div>
                            </div>
                          </div>

                          <span
                            style={{
                              fontSize: '0.72rem',
                              fontWeight: 600,
                              color: '#4ade80',
                              background: 'rgba(34, 197, 94, 0.1)',
                              padding: '2px 8px',
                              borderRadius: '6px',
                            }}
                          >
                            Supported
                          </span>
                        </div>
                      );
                    })}
                </div>
              </div>

              {/* OS System Summary */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '12px 16px',
                  borderRadius: '10px',
                  background: 'rgba(15, 23, 42, 0.4)',
                  border: '1px solid rgba(255, 255, 255, 0.05)',
                  fontSize: '0.8rem',
                  color: '#94a3b8',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Info size={14} style={{ color: '#38bdf8' }} />
                  <span>Sistem Operasi: <strong style={{ color: '#f1f5f9' }}>{hardware.os.toUpperCase()} ({hardware.arch})</strong></span>
                </div>
                {hardware.ffmpeg_path && (
                  <div style={{ fontSize: '0.75rem', opacity: 0.8, overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '280px' }}>
                    Path: {hardware.ffmpeg_path}
                  </div>
                )}
              </div>
            </div>
          ) : null}
        </div>

        <div className="modal-footer" style={{ padding: '16px 24px', borderTop: '1px solid rgba(255, 255, 255, 0.08)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <button
            className="btn btn-secondary"
            onClick={fetchHardware}
            disabled={loading}
            style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '0.85rem' }}
          >
            <RefreshCw size={14} className={loading ? 'spin' : ''} />
            Pindai Ulang Hardware
          </button>
          <button className="btn btn-primary" onClick={onClose} style={{ minWidth: '100px' }}>
            Tutup
          </button>
        </div>
      </div>
    </div>
  );
};
