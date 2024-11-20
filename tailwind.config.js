module.exports = {
  mode: "jit",
  content: {
    files: ["src/**/*.rs", "index.html"],
  },
  theme: {
    extend: {
      colors: {
        gray: {
          900: "#121826",
          800: "#1F2937",
          700: "#374151",
          400: "#9CA3AF",
          300: "#D1D5DB",
        },
        blue: {
          600: "#2563EB",
          700: "#1D4ED8",
        },
      },
    },
  },
  plugins: [],
};
