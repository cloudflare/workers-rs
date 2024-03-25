addEventListener("fetch", (event) => {
    event.respondWith(handleRequest(event.request));
});

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest() {
    const { start } = wasm_bindgen;
    await wasm_bindgen(wasm);

    try {
        const text = await start();

        return new Response(text, {
            status: 200,
            headers: {
                "Content-type": "application/json",
            },
        });
    } catch (error) {
        return new Response(error, {
            status: 500,
        });
    }
}
