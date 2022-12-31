export default {
  fetch: async (...args) => {
    init();

    const imports = require("./index_bg.js");

    // Run the worker's initialization function.
    imports.start?.();

    return imports.fetch(...args);
  },
};
