addEventListener('fetch', event => {
  const { type, request } = event
  event.respondWith(handleRequest(type, request))
})

addEventListener('scheduled', event => {
  const { type, schedule, cron } = event
  event.waitUntil(handleScheduled(type, schedule, cron))
})

async function handleRequest(type, request) {
  const { main } = wasm_bindgen;
  await wasm_bindgen(wasm)

  return main(type, request)
}

async function handleScheduled(type, schedule, cron) {
  const { job } = wasm_bindgen;
  await wasm_bindgen(wasm)

  return job(type, schedule, cron)
}
