import js from "@eslint/js";
import ts from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import jsxA11y from "eslint-plugin-jsx-a11y";
import importPlugin from "eslint-plugin-import";
import globals from "globals";

export default ts.config(
    js.configs.recommended,
    ...ts.configs.recommended,
    {
        files: ["**/*.{ts,tsx}"],
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            globals: {
                ...globals.browser,
                ...globals.es2021,
            },
            parserOptions: {
                ecmaFeatures: {
                    jsx: true,
                },
            },
        },
        plugins: {
            react,
            "react-hooks": reactHooks,
            "jsx-a11y": jsxA11y,
            import: importPlugin,
        },
        settings: {
            react: {
                version: "detect",
            },
            "import/resolver": {
                typescript: true,
                node: true,
            },
        },
        rules: {
            ...react.configs.recommended.rules,
            ...reactHooks.configs.recommended.rules,
            ...jsxA11y.configs.recommended.rules,
            "react/react-in-jsx-scope": "off",
            "@typescript-eslint/no-explicit-any": "warn",
            "@typescript-eslint/no-unused-vars": ["error", {
                "argsIgnorePattern": "^_",
                "varsIgnorePattern": "^_"
            }],
            "jsx-a11y/no-autofocus": "off",
        },
    },
    {
        files: ["electron/**/*.js", "electron/**/*.cjs"],
        languageOptions: {
            globals: {
                ...globals.node,
            },
        },
        rules: {
            "@typescript-eslint/no-require-imports": "off",
        },
    },
    {
        ignores: ["dist", "release", "node_modules", ".github", "*.config.js", "*.config.ts", "assets"],
    }
);
