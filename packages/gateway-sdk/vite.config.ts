import {resolve} from "path";
import {defineConfig} from "vite";
import dts from "vite-plugin-dts";

export default defineConfig({
  plugins: [
    dts({
      outDir: ["dist"],
      exclude: ["vite.config.ts"],
      rollupTypes: true,
      clearPureImport: true,
    }),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, "lib/index.ts"),
      fileName: () => "index.js",
      formats: ["es"],
    },
    rollupOptions: {
      external: [],
    },
  },
  resolve: {
    alias: {
      "~": resolve(__dirname, "./lib"),
    },
  },
});
