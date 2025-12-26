/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "hsl(0, 0%, 7%)",
        foreground: "hsl(0, 0%, 95%)",
        card: {
          DEFAULT: "hsl(0, 0%, 10%)",
          foreground: "hsl(0, 0%, 95%)",
        },
        popover: {
          DEFAULT: "hsl(0, 0%, 10%)",
          foreground: "hsl(0, 0%, 95%)",
        },
        primary: {
          DEFAULT: "hsl(346, 100%, 68%)",
          foreground: "hsl(0, 0%, 98%)",
        },
        secondary: {
          DEFAULT: "hsl(0, 0%, 14%)",
          foreground: "hsl(0, 0%, 95%)",
        },
        muted: {
          DEFAULT: "hsl(0, 0%, 14%)",
          foreground: "hsl(0, 0%, 60%)",
        },
        accent: {
          DEFAULT: "hsl(0, 0%, 14%)",
          foreground: "hsl(0, 0%, 95%)",
        },
        destructive: {
          DEFAULT: "hsl(0, 84%, 60%)",
          foreground: "hsl(0, 0%, 98%)",
        },
        border: "hsl(0, 0%, 18%)",
        input: "hsl(0, 0%, 18%)",
        ring: "hsl(346, 100%, 68%)",
      },
      borderRadius: {
        lg: "0.75rem",
        md: "0.5rem",
        sm: "0.25rem",
      },
    },
  },
  plugins: [],
}
