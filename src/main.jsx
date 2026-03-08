import React from 'react';
import { createRoot } from 'react-dom/client';
import { ThemeProvider } from './ThemeContext';
import App from './App';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { error: null };
  }
  static getDerivedStateFromError(error) {
    return { error };
  }
  render() {
    if (this.state.error) {
      return (
        <div style={{ padding: 40, fontFamily: 'monospace', color: '#ff3b30', background: '#1e1e1e', minHeight: '100vh' }}>
          <h2 style={{ color: '#e5e5e7' }}>Something went wrong</h2>
          <pre style={{ marginTop: 16, whiteSpace: 'pre-wrap', fontSize: 13 }}>
            {this.state.error.toString()}
            {'\n\n'}
            {this.state.error.stack}
          </pre>
        </div>
      );
    }
    return this.props.children;
  }
}

const root = createRoot(document.getElementById('root'));
root.render(
  <ErrorBoundary>
    <ThemeProvider>
      <App />
    </ThemeProvider>
  </ErrorBoundary>
);
