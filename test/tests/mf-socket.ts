import { Miniflare } from "miniflare";
import { readFileSync } from "node:fs";
import { createServer } from "net";

export const server = createServer(function (socket) {
    socket.on('data', function (data) {
        socket.write(data, (err) => {
            console.error(err);
        });
    });
}).listen(8080);

const manifest = {
  mainModule: "build/index.js",
  modules: {
    "build/index.js": {
      type: "esm" as const,
      contents: readFileSync("./build/index.js", "utf8"),
    },
    "build/index_bg.wasm": {
      type: "wasm" as const,
      contents: new Uint8Array(readFileSync("./build/index_bg.wasm")),
    },
  },
};

export const mf = new Miniflare({
  workers: [{
    config: {
      name: "socket-test",
      compatibilityDate: "2024-12-05",
      manifest,
      triggers: [{ type: "connect", protocol: "tcp", port: 0 }],
    },
  }],
});

export const mfUrl = await mf.ready;

export const mfExperimental = new Miniflare({
  workers: [{
    config: {
      name: "socket-test-experimental",
      compatibilityDate: "2024-12-05",
      compatibilityFlags: ["experimental"],
      manifest,
      triggers: [
        { type: "connect", protocol: "tcp", port: 0 },
        { type: "connect", protocol: "udp", port: 0 },
      ],
    },
  }],
});

await mfExperimental.ready;
