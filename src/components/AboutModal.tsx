import React, { useState } from 'react';
import { Activity, X, Globe, ShieldCheck, RefreshCw, CheckCircle2, ArrowUpCircle } from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { checkForUpdates, UpdateCheckResult, CURRENT_VERSION } from '../services/updateService';

interface AboutModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AboutModal: React.FC<AboutModalProps> = ({ isOpen, onClose }) => {
  const [isChecking, setIsChecking] = useState<boolean>(false);
  const [updateResult, setUpdateResult] = useState<UpdateCheckResult | null>(null);

  if (!isOpen) return null;

  const handleOpenWebsite = async (url: string = 'https://crediblemark.com') => {
    try {
      await openUrl(url);
    } catch {
      window.open(url, '_blank');
    }
  };

  const handleCheckForUpdates = async () => {
    setIsChecking(true);
    setUpdateResult(null);
    try {
      const result = await checkForUpdates();
      setUpdateResult(result);
    } catch {
      setUpdateResult({
        hasUpdate: false,
        currentVersion: CURRENT_VERSION,
        latestVersion: CURRENT_VERSION,
        error: 'Tidak dapat memeriksa pembaruan saat ini.',
      });
    } finally {
      setIsChecking(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-container about-modal"
        onClick={(e) => e.stopPropagation()}
        style={{ maxWidth: 480, width: '92%' }}
      >
        <div className="modal-header">
          <div className="brand" style={{ gap: 10 }}>
            <div className="brand-icon" style={{ width: 36, height: 36, borderRadius: 10 }}>
              <Activity className="icon-pulse" size={22} />
            </div>
            <div>
              <h3 style={{ margin: 0, fontSize: 18, fontWeight: 700, color: '#fff' }}>AudioWave Studio</h3>
              <span style={{ fontSize: 12, color: 'var(--accent-cyan)', fontWeight: 600 }}>v{CURRENT_VERSION}</span>
            </div>
          </div>
          <button className="btn-icon" onClick={onClose} title="Tutup">
            <X size={18} />
          </button>
        </div>

        <div className="modal-body" style={{ padding: '20px 24px', display: 'flex', flexDirection: 'column', gap: 16 }}>
          <p style={{ margin: 0, fontSize: 13, lineHeight: 1.6, color: '#cbd5e1' }}>
            High-Performance Desktop Audio Visualizer Studio & Video Generator built with Tauri v2, React 19, and Rust GPU rendering engine.
          </p>

          <div
            style={{
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid rgba(255, 215, 0, 0.15)',
              borderRadius: 12,
              padding: 16,
              display: 'flex',
              flexDirection: 'column',
              gap: 12,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>Versi Aplikasi</span>
              <span style={{ fontSize: 13, color: '#ffd700', fontWeight: 700 }}>v{CURRENT_VERSION}</span>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>Pengembang</span>
              <strong style={{ fontSize: 13, color: '#ffd700', fontWeight: 600 }}>CredibleMark</strong>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>Website Resmi</span>
              <a
                href="https://crediblemark.com"
                onClick={(e) => { e.preventDefault(); handleOpenWebsite('https://crediblemark.com'); }}
                style={{
                  fontSize: 13,
                  color: '#ffd700',
                  textDecoration: 'none',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 4,
                  fontWeight: 500,
                }}
              >
                <Globe size={14} />
                <span>crediblemark.com</span>
              </a>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>Hak Cipta</span>
              <span style={{ fontSize: 12, color: '#a3a3a3' }}>© 2026 CredibleMark</span>
            </div>
          </div>

          {/* UPDATE STATUS SECTION */}
          <div
            style={{
              background: 'rgba(255, 215, 0, 0.04)',
              border: '1px solid rgba(255, 215, 0, 0.2)',
              borderRadius: 10,
              padding: 14,
              display: 'flex',
              flexDirection: 'column',
              gap: 10,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <RefreshCw size={16} style={{ color: '#ffd700' }} className={isChecking ? 'spin-icon' : ''} />
                <span style={{ fontSize: 13, fontWeight: 600, color: '#f8fafc' }}>Pembaruan Perangkat Lunak</span>
              </div>
              <button
                className="btn btn-secondary"
                style={{ padding: '4px 10px', fontSize: 12 }}
                onClick={handleCheckForUpdates}
                disabled={isChecking}
              >
                {isChecking ? 'Memeriksa...' : 'Cek Pembaruan'}
              </button>
            </div>

            {updateResult && (
              <div style={{ fontSize: 12, marginTop: 4 }}>
                {updateResult.hasUpdate ? (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    <div style={{ color: '#ffd700', display: 'flex', alignItems: 'center', gap: 6, fontWeight: 600 }}>
                      <ArrowUpCircle size={16} />
                      <span>Versi terbaru v{updateResult.latestVersion} tersedia!</span>
                    </div>
                    <button
                      className="btn btn-primary"
                      style={{ fontSize: 12, padding: '6px 12px' }}
                      onClick={() => handleOpenWebsite(updateResult.downloadUrl || 'https://crediblemark.com')}
                    >
                      Unduh Pembaruan v{updateResult.latestVersion}
                    </button>
                  </div>
                ) : updateResult.error ? (
                  <span style={{ color: '#ff6b6b' }}>{updateResult.error}</span>
                ) : (
                  <div style={{ color: '#4ade80', display: 'flex', alignItems: 'center', gap: 6 }}>
                    <CheckCircle2 size={16} />
                    <span>Anda sudah menggunakan versi terbaru (v{CURRENT_VERSION}).</span>
                  </div>
                )}
              </div>
            )}
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, color: 'var(--text-muted)' }}>
            <ShieldCheck size={14} style={{ color: '#ffd700' }} />
            <span>Hak cipta dilindungi undang-undang. Powered by CredibleMark.</span>
          </div>
        </div>

        <div className="modal-footer" style={{ justifyContent: 'space-between' }}>
          <button className="btn btn-secondary" onClick={() => handleOpenWebsite('https://crediblemark.com')}>
            <Globe size={14} />
            <span>Kunjungi Website</span>
          </button>
          <button className="btn btn-primary" onClick={onClose}>
            Tutup
          </button>
        </div>
      </div>
    </div>
  );
};
