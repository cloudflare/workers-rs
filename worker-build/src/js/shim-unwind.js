import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

Error.stackTraceLimit = 100;

addEventListener('error', (e) => {
  if (e.error instanceof WebAssembly.RuntimeError) {
    console.error('Critical', e.error);
  }
});

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

export default Entrypoint;
