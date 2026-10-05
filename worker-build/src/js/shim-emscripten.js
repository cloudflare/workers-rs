import { DurableObject, WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

export default Entrypoint;
