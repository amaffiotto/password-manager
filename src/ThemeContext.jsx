import React, { createContext, useContext, useState, useEffect } from 'react';

const ThemeContext = createContext();

const DEFAULTS = {
  accentColor: '',
  bgColor: '',
  uiOpacity: 1,
};

function loadAppearance() {
  try {
    const raw = localStorage.getItem('pm-appearance');
    return raw ? { ...DEFAULTS, ...JSON.parse(raw) } : { ...DEFAULTS };
  } catch {
    return { ...DEFAULTS };
  }
}

function applyAppearance(appearance) {
  const root = document.documentElement;
  if (appearance.accentColor) {
    root.style.setProperty('--accent', appearance.accentColor);
    root.style.setProperty('--accent-hover', appearance.accentColor);
    root.style.setProperty('--bg-selected', appearance.accentColor);
  } else {
    root.style.removeProperty('--accent');
    root.style.removeProperty('--accent-hover');
    root.style.removeProperty('--bg-selected');
  }
  if (appearance.bgColor) {
    root.style.setProperty('--bg-primary', appearance.bgColor);
  } else {
    root.style.removeProperty('--bg-primary');
  }
  root.style.setProperty('--ui-opacity', appearance.uiOpacity);
}

export function ThemeProvider({ children }) {
  const [theme, setTheme] = useState(() => {
    try {
      return localStorage.getItem('pm-theme') || 'dark';
    } catch {
      return 'dark';
    }
  });

  const [appearance, setAppearanceState] = useState(loadAppearance);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    try {
      localStorage.setItem('pm-theme', theme);
    } catch {}
    setTimeout(() => applyAppearance(appearance), 0);
  }, [theme]);

  useEffect(() => {
    applyAppearance(appearance);
    try {
      localStorage.setItem('pm-appearance', JSON.stringify(appearance));
    } catch {}
  }, [appearance]);

  const toggleTheme = () => setTheme((t) => (t === 'dark' ? 'light' : 'dark'));

  const setAppearance = (updates) => {
    setAppearanceState((prev) => ({ ...prev, ...updates }));
  };

  const resetAppearance = () => {
    setAppearanceState({ ...DEFAULTS });
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme, appearance, setAppearance, resetAppearance }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext);
}
