import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{svelte,ts}"],
  theme: {
    extend: {
      colors: {
        chrome: {
          950: "#0d1117",
          900: "#131a22",
          800: "#1a2330",
          700: "#233044",
          600: "#31415c",
          300: "#95a7bf",
          200: "#c4d1e3",
          100: "#e5edf9"
        },
        accent: {
          500: "#58c4dc",
          400: "#7cd8eb"
        },
        signal: {
          success: "#4ade80",
          warning: "#fbbf24",
          danger: "#fb7185"
        }
      },
      boxShadow: {
        overlay: "0 18px 60px rgba(0, 0, 0, 0.45)"
      },
      fontFamily: {
        sans: ["'IBM Plex Sans'", "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ["'IBM Plex Mono'", "ui-monospace", "monospace"]
      }
    }
  },
  plugins: []
};

export default config;
