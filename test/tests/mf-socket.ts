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

export const mf = new Miniflare({
  workers: [{
    config: {
      name: "socket-test",
      type: "worker",
      compatibilityDate: "2024-12-05",
      manifest: {
        mainModule: "build/index.js",
        modules: {
          "build/index.js": {
            type: "esm",
            contents: readFileSync("./build/index.js", "utf8"),
          },
          "build/index_bg.wasm": {
            type: "wasm",
            contents: new Uint8Array(readFileSync("./build/index_bg.wasm")),
          },
        },
      },
    },
  }],
});

export const mfUrl = await mf.ready;
