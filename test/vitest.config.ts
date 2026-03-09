import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    exclude: ["**/panic-unwind/**"],
  },
});
