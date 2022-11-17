addEventListener('fetch', event => {
    event.respondWith(new Response('hello world'))
})
