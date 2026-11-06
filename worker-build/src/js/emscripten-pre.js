// AUTO_INIT takes no Module hooks such as `onAbort`, so `abort` itself reports to the shim.
var workersRsAbort = abort;
abort = (what) => {
  globalThis[Symbol.for("workers-rs.abort")]?.(what);
  return workersRsAbort(what);
};
