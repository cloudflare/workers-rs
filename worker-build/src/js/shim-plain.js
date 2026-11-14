import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index_bg.js";

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

export default Entrypoint;
