import { defineConfig } from "vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { nitro } from "nitro/vite";
import { pwaPlugin } from "./scripts/pwa-plugin.ts";
import { appEnvPlugin } from "./scripts/app-env-plugin.ts";

const devPublicHost = process.env.PDFIN_DEV_PUBLIC_HOST?.trim();

const apiProxy = {
  "/rust-api": {
    target: "http://127.0.0.1:3000",
    changeOrigin: true,
  },
} as const;

const publicHostOptions = devPublicHost
  ? { allowedHosts: [devPublicHost] }
  : {};

export default defineConfig(({ command, isPreview }) => ({
  server: {
    host: "127.0.0.1",
    port: 8080,
    strictPort: true,
    ...publicHostOptions,
    proxy: apiProxy,
  },
  preview: {
    host: "127.0.0.1",
    port: 8081,
    strictPort: true,
    ...publicHostOptions,
    proxy: apiProxy,
  },
  resolve: { tsconfigPaths: true },
  plugins: [
    appEnvPlugin(),
    pwaPlugin(),
    tailwindcss(),
    tanstackStart(),
    ...(command === "build" || isPreview
      ? [
          nitro({
            preset: "vercel",
            serverDir: "./server",
          }),
        ]
      : []),
    viteReact(),
  ],
  test: {
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    exclude: ["scripts/**/*.test.{mjs,ts}"],
  },
}));
