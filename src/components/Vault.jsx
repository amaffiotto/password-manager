import React from 'react';

const ICON_COLORS = ['icon-blue', 'icon-green', 'icon-orange', 'icon-purple', 'icon-teal', 'icon-pink', 'icon-red', 'icon-indigo'];

function getIconColor(name) {
  let hash = 0;
  for (let i = 0; i < name.length; i++) hash = name.charCodeAt(i) + ((hash << 5) - hash);
  return ICON_COLORS[Math.abs(hash) % ICON_COLORS.length];
}

function formatModified(dateStr) {
  if (!dateStr) return '';
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now - d;
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;

    return d.toLocaleDateString(undefined, {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  } catch {
    return dateStr;
  }
}

export default function Vault({ entries, selectedEntry, onSelect, favourites }) {
  return (
    <div className="entry-table-container">
      <div className="entry-table-scroll">
        <table className="entry-table">
          <thead className="entry-table-header">
            <tr>
              <th className="entry-icon-cell"></th>
              <th className="col-title">Title</th>
              <th className="col-username">Username</th>
              <th className="col-url">URL</th>
              <th className="col-modified">Modified</th>
            </tr>
          </thead>
          <tbody className="entry-table-body">
            {entries.length === 0 ? (
              <tr>
                <td colSpan="5" style={{ textAlign: 'center', padding: '48px', color: 'var(--text-tertiary)' }}>
                  No entries found. Click + to add one.
                </td>
              </tr>
            ) : (
              entries.map((entry) => {
                const isSelected = selectedEntry && selectedEntry.id === entry.id;
                const isFav = favourites.has(entry.id);
                const iconColor = getIconColor(entry.site_name);
                const initial = entry.site_name.charAt(0).toUpperCase();

                return (
                  <tr
                    key={entry.id}
                    className={isSelected ? 'selected' : ''}
                    onClick={() => onSelect(entry)}
                  >
                    <td className="entry-icon-cell">
                      <div className={`entry-icon ${iconColor}`}>{initial}</div>
                    </td>
                    <td>
                      {entry.site_name}
                      {isFav && <span className="entry-fav">{'\u2605'}</span>}
                    </td>
                    <td>{entry.username}</td>
                    <td>{entry.url || ''}</td>
                    <td>{formatModified(entry.updated_at)}</td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
