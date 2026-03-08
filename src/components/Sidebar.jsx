import React from 'react';

const ICON_COLORS = ['icon-blue', 'icon-green', 'icon-orange', 'icon-purple', 'icon-teal', 'icon-pink', 'icon-red', 'icon-indigo'];

function getIconColor(name) {
  let hash = 0;
  for (let i = 0; i < name.length; i++) hash = name.charCodeAt(i) + ((hash << 5) - hash);
  return ICON_COLORS[Math.abs(hash) % ICON_COLORS.length];
}

export default function Sidebar({ entries, view, onViewChange, favourites }) {
  const favEntries = entries.filter((e) => favourites.has(e.id));
  const totalEntries = entries.length;
  const weakEntries = entries.filter((e) => e._weak).length;

  return (
    <div className="sidebar">
      {/* Favourites */}
      {favEntries.length > 0 && (
        <div className="sidebar-section">
          <div className="sidebar-section-header">Favourites</div>
          {favEntries.map((entry) => (
            <div
              key={`fav-${entry.id}`}
              className={`sidebar-item ${view === `fav-${entry.id}` ? 'active' : ''}`}
              onClick={() => onViewChange(`fav-${entry.id}`)}
            >
              <span className="sidebar-icon">&#9733;</span>
              <span className="sidebar-label">{entry.site_name}</span>
            </div>
          ))}
        </div>
      )}

      {/* Hierarchy */}
      <div className="sidebar-section">
        <div className="sidebar-section-header">Hierarchy</div>
        <div
          className={`sidebar-item ${view === 'vault' ? 'active' : ''}`}
          onClick={() => onViewChange('vault')}
        >
          <span className="sidebar-icon">&#128274;</span>
          <span className="sidebar-label">Database</span>
          <span className="sidebar-count">({totalEntries})</span>
        </div>
      </div>

      {/* Tags */}
      <div className="sidebar-section">
        <div className="sidebar-section-header">Tags</div>
        <div
          className={`sidebar-item ${view === 'pgp' ? 'active' : ''}`}
          onClick={() => onViewChange('pgp')}
        >
          <span className="sidebar-icon">&#128273;</span>
          <span className="sidebar-label">PGP Keys</span>
        </div>
        <div
          className={`sidebar-item ${view === 'favourites' ? 'active' : ''}`}
          onClick={() => onViewChange('favourites')}
        >
          <span className="sidebar-icon">&#9733;</span>
          <span className="sidebar-label">Favourite</span>
        </div>
        <div
          className={`sidebar-item ${view === 'settings' ? 'active' : ''}`}
          onClick={() => onViewChange('settings')}
        >
          <span className="sidebar-icon">&#9881;</span>
          <span className="sidebar-label">Settings</span>
        </div>
      </div>

      <div className="sidebar-divider" />

      {/* Quick Views */}
      <div className="sidebar-section">
        <div className="sidebar-section-header">Quick Views</div>
        <div
          className={`sidebar-item ${view === 'all' ? 'active' : ''}`}
          onClick={() => onViewChange('all')}
        >
          <span className="sidebar-icon">&#128196;</span>
          <span className="sidebar-label">All Entries</span>
        </div>
      </div>
    </div>
  );
}
