import { Miniflare } from "miniflare";
import { createServer } from "net";
import { manifest } from "./manifest";

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
