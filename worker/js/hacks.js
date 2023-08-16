/**
 * @param {Request | Response} r
 * @returns {Promise<ArrayBuffer>}
 */
export async function collectBytes(r) {
  return await new Response(
    r.body.pipeThrough(
      new TransformStream({
        transform(chunk, controller) {
          console.log("TRANSFORM", chunk);
          const cloned = new Uint8Array(chunk);
          controller.enqueue(cloned);
        },
      })
    )
  ).arrayBuffer();
}
