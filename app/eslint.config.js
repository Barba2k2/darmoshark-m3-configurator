import js from "@eslint/js";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist", "src-tauri"] },
  {
    files: ["**/*.{ts,tsx}"],
    extends: [js.configs.recommended, ...tseslint.configs.strict],
    languageOptions: { globals: globals.browser },
    plugins: { "react-hooks": reactHooks },
    rules: {
      ...reactHooks.configs.recommended.rules,
      // Project convention: constants, routes and helpers are classes with
      // static members, never loose `export const` values or functions.
      "@typescript-eslint/no-extraneous-class": "off",
      "no-restricted-imports": [
        "error",
        { paths: [{ name: "react", importNames: ["useState"], message: "State lives in Zustand stores." }] },
      ],
    },
  },
);
