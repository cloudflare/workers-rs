export function capabilityFixture() {
  const fixture = { chunks: [], completed: false };
  fixture.input = new ReadableStream({
    type: "bytes",
    start(controller) { fixture.controller = controller; },
  });
  fixture.props = { scope: "synthetic-a", capability: { marker: "preserved" } };
  const exports = {};
  exports.ScopedService = new Proxy(function ({ props }) {
    if (this !== exports) throw new Error("factory receiver lost");
    fixture.lastProps = props;
    return {
      fetch(request) {
        fixture.request = request;
        return Promise.resolve(new Response(props.scope));
      },
    };
  }, {
    get(target, property, receiver) {
      if (property === "call" || property === "apply") {
        throw new Error("RPC method trap must not be used");
      }
      return Reflect.get(target, property, receiver);
    },
  });
  fixture.context = { exports };
  fixture.state = { exports };
  fixture.container = {
    running: true,
    setInactivityTimeout(duration) {
      if (this !== fixture.container) throw new Error("timeout receiver lost");
      if (duration === 0) throw new TypeError("synthetic timeout must be positive");
      if (duration > 21600000) return Promise.reject(new Error("synthetic timeout exceeds six hours"));
      fixture.timeout = duration;
      return new Promise(resolve => { fixture.finishTimeout = resolve; });
    },
    images: Object.freeze({ base: "registry.example.invalid/synthetic@sha256:0123456789" }),
    interceptOutboundHttp(target, handler) {
      if (this !== fixture.container) throw new Error("container receiver lost");
      if (target === "throw.invalid:4000") throw new Error("synthetic sync intercept error");
      if (target === "reject.invalid:4000") return Promise.reject(new Error("synthetic async intercept error"));
      fixture.target = target;
      fixture.handler = handler;
      return new Promise(resolve => { fixture.install = resolve; });
    },
    exec(command, options) {
      fixture.command = command;
      fixture.options = options;
      let input = options?.stdin;
      let stdin;
      if (input === "pipe") {
        const pipe = new TransformStream();
        input = pipe.readable;
        stdin = pipe.writable;
        fixture.pipe = stdin;
      }
      const output = (async () => {
        const bytes = [];
        if (input) {
          const reader = input.getReader();
          for (;;) {
            const { done, value } = await reader.read();
            if (done) break;
            fixture.chunks.push(Array.from(value));
            bytes.push(...value);
          }
        }
        fixture.completed = true;
        return { stdout: Uint8Array.from(bytes).buffer, stderr: new ArrayBuffer(0), exitCode: 0 };
      })();
      const byteStream = bytes => new ReadableStream({
        type: "bytes",
        start(controller) { controller.enqueue(Uint8Array.from(bytes)); controller.close(); },
      });
      fixture.stdout = options?.stdout === "ignore" ? undefined : byteStream([5, 0, 255]);
      fixture.stderr = ["ignore", "combined"].includes(options?.stderr) ? undefined : byteStream([6, 128]);
      return Promise.resolve({
        pid: 42, stdin, stdout: fixture.stdout, stderr: fixture.stderr,
        output: () => output,
      });
    },
  };
  return fixture;
}

export function fixtureInput(fixture) { return fixture.input; }
export function fixtureContainer(fixture) { return fixture.container; }
export function fixtureState(fixture) { return fixture.state; }
export function fixtureContext(fixture) { return fixture.context; }
export function fixtureProps(fixture) { return fixture.props; }
export function pushInput(fixture, bytes) { fixture.controller.enqueue(Uint8Array.from(bytes)); }
export function closeInput(fixture) { fixture.controller.close(); }
export function failInput(fixture) { fixture.controller.error(new Error("synthetic stream error")); }
export async function tick() { await new Promise(resolve => setTimeout(resolve, 0)); }
export function inputIdentityPreserved(fixture) { return fixture.options.stdin === fixture.input; }
export function chunks(fixture) { return JSON.stringify(fixture.chunks); }
export function completed(fixture) { return fixture.completed; }
export function finishIntercept(fixture) { fixture.install(); }
export function interceptTarget(fixture) { return fixture.target; }
export function handlerIdentityPreserved(fixture, handler) { return fixture.handler === handler; }
export function propsIdentityPreserved(fixture) { return fixture.lastProps === fixture.props; }
export function setScope(fixture, value) { fixture.props = { scope: value, capability: fixture.props.capability }; }
export function pipeIdentityPreserved(fixture, pipe) { return fixture.pipe === pipe; }
export async function writePipe(pipe) {
  const writer = pipe.getWriter();
  await writer.write(new Uint8Array([9, 0, 255]));
  await writer.close();
}
export function throwingImages(fixture) {
  Object.defineProperty(fixture.container, "images", { get() { throw new Error("synthetic images error"); } });
}
export function throwingFactory(fixture) {
  fixture.state.exports.ScopedService = () => { throw new Error("synthetic factory error"); };
}
export function outputIdentityPreserved(fixture, stdout, stderr) {
  return fixture.stdout === stdout && fixture.stderr === stderr;
}
export async function streamBytes(stream) { return new Uint8Array(await new Response(stream).arrayBuffer()); }
export function inactivityTimeout(fixture) { return fixture.timeout; }
export function finishTimeout(fixture) { fixture.finishTimeout(); }
