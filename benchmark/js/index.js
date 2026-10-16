const CHUNK = new Uint8Array(8192).fill(0x78);

export default {
  async fetch(request, env) {
    switch (new URL(request.url).pathname) {
      case "/hello":
        return new Response("hello");
      case "/headers": {
        const h = request.headers;
        return new Response(null, {
          headers: {
            "x-o1": h.get("x-h1") ?? "",
            "x-o2": h.get("x-h2") ?? "",
            "x-o3": h.get("x-h3") ?? "",
            "x-o4": h.get("x-h4") ?? "",
            "x-o5": h.get("x-h5") ?? "",
          },
        });
      }
      case "/echo":
        return new Response(await request.text());
      case "/json": {
        const payload = await request.json();
        payload.id += 1;
        return Response.json(payload);
      }
      case "/kv": {
        const value = (await env.KV.get("bench")) ?? "v".repeat(1024);
        await env.KV.put("bench", value);
        return new Response(value);
      }
      case "/stream": {
        let sent = 0;
        return new Response(
          new ReadableStream({
            pull(controller) {
              if (sent++ < 128) controller.enqueue(CHUNK.slice());
              else controller.close();
            },
          }),
        );
      }
      case "/fanout": {
        const bodies = await Promise.all(
          Array.from({ length: 10 }, () =>
            env.SERVICE.fetch("http://bench/hello").then((res) => res.text()),
          ),
        );
        return new Response(String(bodies.reduce((n, body) => n + body.length, 0)));
      }
      default:
        return new Response("Not Found", { status: 404 });
    }
  },
};
