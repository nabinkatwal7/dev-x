import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{svelte,ts}"],
  theme: {
    extend: {
      colors: {
        chrome: {
          950: "rgb(var(--chrome-950) / <alpha-value>)",
          900: "rgb(var(--chrome-900) / <alpha-value>)",
          800: "rgb(var(--chrome-800) / <alpha-value>)",
          700: "rgb(var(--chrome-700) / <alpha-value>)",
          600: "rgb(var(--chrome-600) / <alpha-value>)",
          300: "rgb(var(--chrome-300) / <alpha-value>)",
          200: "rgb(var(--chrome-200) / <alpha-value>)",
          100: "rgb(var(--chrome-100) / <alpha-value>)"
        },
        accent: {
          500: "rgb(var(--accent-500) / <alpha-value>)",
          400: "rgb(var(--accent-400) / <alpha-value>)"
        },
        signal: {
          success: "rgb(var(--signal-success) / <alpha-value>)",
          warning: "rgb(var(--signal-warning) / <alpha-value>)",
          danger: "rgb(var(--signal-danger) / <alpha-value>)"
        }
      },
      boxShadow: {
        overlay: "0 18px 60px rgba(0, 0, 0, 0.45)"
      },
      fontFamily: {
        sans: [
          "'Fira Code'",
          "'SF Mono'",
          "'Cascadia Code'",
          "'JetBrains Mono'",
          "'Consolas'",
          "ui-monospace",
          "monospace"
        ],
        mono: [
          "'Fira Code'",
          "'SF Mono'",
          "'Cascadia Code'",
          "'JetBrains Mono'",
          "'Consolas'",
          "ui-monospace",
          "monospace"
        ]
      }
    }
  },
  plugins: []
};

export default config;
