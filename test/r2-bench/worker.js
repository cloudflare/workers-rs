// Pure JS worker to test R2 parallel vs sequential get performance.
// Run with: cd test/r2-bench && npx wrangler dev

const COUNT = 1024;

async function seed(bucket) {
  const existing = await bucket.head(`bench/key-0`);
  if (existing) return;
  for (let i = 0; i < COUNT; i++) {
    await bucket.put(`bench/key-${i}`, `value-${i}`);
  }
}

async function sequential(bucket) {
  const start = Date.now();
  const values = [];
  for (let i = 0; i < COUNT; i++) {
    const obj = await bucket.get(`bench/key-${i}`);
    values.push(await obj.text());
  }
  return { mode: "sequential", count: values.length, elapsed_ms: Date.now() - start };
}

async function parallel(bucket) {
  const start = Date.now();
  const promises = [];
  for (let i = 0; i < COUNT; i++) {
    promises.push(
      bucket.get(`bench/key-${i}`).then((obj) => obj.text())
    );
  }
  const values = await Promise.all(promises);
  return { mode: "parallel", count: values.length, elapsed_ms: Date.now() - start };
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const bucket = env.BUCKET;

    if (url.pathname === "/seed") {
      await seed(bucket);
      return Response.json({ seeded: COUNT });
    }

    if (url.pathname === "/sequential") {
      await seed(bucket);
      const result = await sequential(bucket);
      return Response.json(result);
    }

    if (url.pathname === "/parallel") {
      await seed(bucket);
      const result = await parallel(bucket);
      return Response.json(result);
    }

    if (url.pathname === "/both") {
      await seed(bucket);
      const seq = await sequential(bucket);
      const par = await parallel(bucket);
      return Response.json({ sequential: seq, parallel: par });
    }

    return new Response("GET /sequential, /parallel, or /both", { status: 404 });
  },
};
