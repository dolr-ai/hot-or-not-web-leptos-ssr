/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./ssr/*.html", "./ssr/src/**/*.rs"],
  theme: {
    extend: {
      colors: {
        primary: {
          50: "rgb(var(--color-primary-50))",
          100: "rgb(var(--color-primary-100))",
          200: "rgb(var(--color-primary-200))",
          300: "rgb(var(--color-primary-300))",
          400: "rgb(var(--color-primary-400))",
          500: "rgb(var(--color-primary-500))",
          600: "rgb(var(--color-primary-600))",
          700: "rgb(var(--color-primary-700))",
          800: "rgb(var(--color-primary-800))",
          900: "rgb(var(--color-primary-900))",
          950: "rgb(var(--color-primary-950))",
        },
      },
      keyframes: {
        "blink-colors": {
          "0%": { color: "red" },
          "25%": { color: "lime" },
          "50%": { color: "yellow" },
          "75%": { color: "fuchsia" },
          "100%": { color: "blue" },
        },
        "searching-a-1": {
          "0%, 49%": { opacity: 1 },
          "50%, 100%": { opacity: 0 },
        },
        "searching-a-2": {
          "0%, 49%": { opacity: 0 },
          "50%, 100%": { opacity: 1 },
        },
        'shake' : {
          '0%, 50%, 100%': {
            transform: 'translate(0)'
          },
          '25%' : {
            transform: 'translate(-1px, -1px)'
          },
          '75%': {
            transform: 'translate(-1px, 1px)'
          },
        },
        'push-right': {
          '0%': {
            transform: 'translate(0) rotate(8deg)',
          },
          '25%': {
            transform: 'translate(4px)',
          },
          '50%': {
            transform: 'translate(0)',
          },
          '75%': {
            transform: 'translate(1px)',
          },
          to: {
            transform: 'translate(0) rotate(0)'
          }
        },
        'push-left': {
          '0%': {
            transform: 'translate(0) rotate(-5deg)',
          },
          '25%': {
            transform: 'translate(-4px)',
          },
          '50%': {
            transform: 'translate(0)',
          },
          '75%': {
            transform: 'translate(-1px)',
          },
          to: {
            transform: 'translate(0) rotate(2deg)',
          }
        },
        shimmer: {
          '0%': { backgroundPositionX: '100%' },
          '100%': { backgroundPositionX: '0%' },
        },
        'slide-up': {
          '0%': {
            opacity: '1',
            transform: 'translateY(0%)',
          },
          '10%': {
            opacity: '1',
            transform: 'translateY(-260%)',
          },
          '50%': {
            opacity: '1',
            transform: 'translateY(-260%)',
          },
          '55%': {
            opacity: '1',
            transform: 'translateY(-500%)',
          },
          '59%': {
            opacity: '0',
            transform: 'translateY(-500%)',
          },
          '60%': {
            opacity: '0',
            transform: 'translateY(0%)',
          },
          '100%': {
            opacity: '0',
            transform: 'translateY(0%)',
          },
        },
      },
      animation: {
        "blink-colors": "blink-colors 5s step-end infinite",
        "searching-a-1": "searching-a-1 2s infinite",
        "searching-a-2": "searching-a-2 2s infinite",
        'shake': 'shake 0.2s cubic-bezier(.56, .14, 0, 1.48) forwards',
        'push-right': 'push-right 0.2s cubic-bezier(.56, .14, 0, 1.48) forwards',
        'push-left': 'push-left 0.2s cubic-bezier(.56, .14, 0, 1.48) forwards',
        'skeleton-shimmer': 'shimmer 1200ms linear infinite',
        'slide-up': 'slide-up 10s ease-in-out var(--animation-delay, 0s) infinite',
      },
      backgroundImage: {
        'skeleton-shimmer': 'linear-gradient(-45deg, currentColor 40%, var(--shimmer, #FFFFFF) 50%, currentColor 60%)',
        "brand-gradient": "var(--color-brand-gradient)",
        "brand-gradient-disabled": "var(--color-brand-gradient-disabled)",
      },
      backgroundSize: {
        'skeleton-shimmer': '300%',
      },
    },
  },
  plugins: [],
};
