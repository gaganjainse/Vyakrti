/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        ide: {
          bg: 'var(--ide-bg)',
          panel: 'var(--ide-panel)',
          border: 'var(--ide-border)',
          text: 'var(--ide-text)',
          accent: 'var(--ide-accent)',
          error: 'var(--ide-error)',
          success: 'var(--ide-success)',
          warning: 'var(--ide-warning)',
          muted: 'var(--ide-muted)',
          surface: 'var(--ide-surface)',
          hover: 'var(--ide-hover)',
          active: 'var(--ide-active)',
        }
      },
      fontFamily: {
        mono: ["'Fira Code'", "'Cascadia Code'", "'Noto Sans Devanagari'", "Consolas", "monospace"],
        sans: ["Inter", "system-ui", "sans-serif"],
        devanagari: ["'Noto Sans Devanagari'", "'Sanskrit Text'", "serif"],
      },
      animation: {
        'fade-in': 'fadeIn 0.15s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}
