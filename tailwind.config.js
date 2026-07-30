/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        brand: {
          50: 'var(--brand-50, #f5f3ff)',
          100: 'var(--brand-100, #ede9fe)',
          200: 'var(--brand-200, #ddd6fe)',
          300: 'var(--brand-300, #c4b5fd)',
          400: 'var(--brand-400, #a78bfa)',
          500: 'var(--brand-500, #8b5cf6)',
          600: 'var(--brand-600, #7c3aed)',
          700: 'var(--brand-700, #6d28d9)',
          800: 'var(--brand-800, #4c1d95)',
          900: 'var(--brand-900, #312e81)',
          950: 'var(--brand-950, #1e1b4b)',
        },
        gold: {
          50: 'var(--gold-50, #fefce8)',
          100: 'var(--gold-100, #fef9c3)',
          200: 'var(--gold-200, #fef08a)',
          300: 'var(--gold-300, #fde047)',
          400: 'var(--gold-400, #d4af37)',
          500: 'var(--gold-500, #d4af37)',
          600: 'var(--gold-600, #b8860b)',
          700: 'var(--gold-700, #92400e)',
          800: 'var(--gold-800, #78350f)',
          900: 'var(--gold-900, #451a03)',
        },
        navy: {
          50: 'var(--navy-50, #eef2ff)',
          100: 'var(--navy-100, #e0e7ff)',
          200: 'var(--navy-200, #c7d2fe)',
          300: 'var(--navy-300, #a5b4fc)',
          400: 'var(--navy-400, #818cf8)',
          500: 'var(--navy-500, #6366f1)',
          600: 'var(--navy-600, #4f46e5)',
          700: 'var(--navy-700, #4338ca)',
          800: 'var(--navy-800, #312e81)',
          900: 'var(--navy-900, #1e1b4b)',
        },
        surface: {
          50: 'var(--surface-50, #f8fafc)',
          100: 'var(--surface-100, #f1f5f9)',
          200: 'var(--surface-200, #e2e8f0)',
          300: 'var(--surface-300, #cbd5e1)',
          400: 'var(--surface-400, #94a3b8)',
          500: 'var(--surface-500, #64748b)',
          600: 'var(--surface-600, #475569)',
          700: 'var(--surface-700, #334155)',
          800: 'var(--surface-800, #1e293b)',
          900: 'var(--surface-900, #0f172a)',
          950: 'var(--surface-950, #020617)',
        }
      },
      fontFamily: {
        sans: ['Inter', 'Cairo', 'system-ui', 'sans-serif'],
        display: ['Plus Jakarta Sans', 'Cairo', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      boxShadow: {
        'glow': '0 0 20px rgba(76, 29, 149, 0.15)',
        'glow-gold': '0 0 20px rgba(212, 175, 55, 0.15)',
        'luxury': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
        'card': '0 1px 3px rgba(0,0,0,0.08), 0 8px 32px rgba(0,0,0,0.06)',
        'card-hover': '0 4px 12px rgba(0,0,0,0.12), 0 16px 48px rgba(0,0,0,0.1)',
      },
      borderRadius: {
        'xl': '1rem',
        '2xl': '1.5rem',
        '3xl': '2rem',
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'slide-up': 'slideUp 0.4s ease-out',
        'scale-in': 'scaleIn 0.2s ease-out',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideIn: {
          '0%': { opacity: '0', transform: 'translateX(-10px)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        scaleIn: {
          '0%': { opacity: '0', transform: 'scale(0.95)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
      },
    },
  },
  plugins: [],
};
