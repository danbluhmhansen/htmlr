const { iconsPlugin, getIconCollections } = require("@egoist/tailwindcss-icons");

/** @type {import('tailwindcss').Config} */
export default {
  content: ["src/**/*.rs"],
  theme: {
    extend: {},
  },
  plugins: [
    require('@tailwindcss/forms'),
    iconsPlugin({
      collections: getIconCollections(["tabler"]),
    }),
  ],
};
