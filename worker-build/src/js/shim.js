import * as imports from "./index_bg.js";
export * from "./index_bg.js";

// Run the worker's initialization function.
imports.start?.();

export default { connect: imports.connect, fetch: imports.fetch, scheduled: imports.scheduled, queue: imports.queue };
