import React from 'react';
import { Activity, X, Globe, ShieldCheck } from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';

interface AboutModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AboutModal: React.FC<AboutModalProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  const handleOpenWebsite = async (e: React.MouseEvent) => {
    e.preventDefault();
    try {
      await openUrl('https://crediblemark.com');
    } catch {
      window.open('https://crediblemark.com', '_blank');
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-container about-modal" onClick={(e) => e.stopPropagation()} style={{ maxWidth: 460, width: '90%' }}>
        <div className="modal-header">
          <div className="brand" style={{ gap: 10 }}>
            <div className="brand-icon" style={{ width: 36, height: 36, borderRadius: 10 }}>
              <Activity className="icon-pulse" size={22} />
            </div>
            <div>
              <h3 style={{ margin: 0, fontSize: 18, fontWeight: 700, color: '#fff' }}>AudioWave Studio</h3>
              <span style={{ fontSize: 12, color: 'var(--text-muted)' }}>Version 1.9.0</span>
            </div>
          </div>
          <button className="btn-icon" onClick={onClose} title="Close">
            <X size={18} />
          </button>
        </div>

        <div className="modal-body" style={{ padding: '20px 24px', display: 'flex', flexDirection: 'column', gap: 16 }}>
          <p style={{ margin: 0, fontSize: 13, lineHeight: 1.6, color: '#cbd5e1' }}>
            High-Performance Desktop Audio Visualizer & Video Generator built with Tauri v2, React 19, and Rust GPU rendering engine.
          </p>

          <div style={{ background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.08)', borderRadius: 12, padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>Developer</span>
              <strong style={{ fontSize: 13, color: '#00e5ff', fontWeight: 600 }}>CredibleMark</strong>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>Website</span>
              <a
                href="https://crediblemark.com"
                onClick={handleOpenWebsite}
                style={{
                  fontSize: 13,
                  color: '#00e5ff',
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
              <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>Copyright</span>
              <span style={{ fontSize: 12, color: '#94a3b8' }}>© 2026 CredibleMark</span>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, color: 'var(--text-muted)', marginTop: 4 }}>
            <ShieldCheck size={14} className="text-secondary" />
            <span>All rights reserved. Powered by CredibleMark.</span>
          </div>
        </div>

        <div className="modal-footer" style={{ justifyContent: 'space-between' }}>
          <button className="btn btn-secondary" onClick={handleOpenWebsite}>
            <Globe size={14} />
            <span>Visit Website</span>
          </button>
          <button className="btn btn-primary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
