use crate::Env;
use crate::SomeSharedData;
use js_sys::Uint8Array;
use std::sync::atomic::{AtomicUsize, Ordering};
use ts_gen_worker::*;

/// Connections handled by this Wasm instance; resets if the instance is reinitialized.
static CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Reads from a stream until `buf` is full; the leftover of the final chunk is dropped
/// since the handlers below only ever read once.
async fn read_exact(
    reader: &ReadableStreamDefaultReader<Uint8Array>,
    buf: &mut [u8],
) -> Result<()> {
    let mut filled = 0;
    while filled < buf.len() {
        let chunk = reader.read().await?;
        if chunk.done() {
            return Err("socket closed before the request was read".into());
        }
        let bytes = chunk.value().unwrap_or_default().to_vec();
        let n = bytes.len().min(buf.len() - filled);
        buf[filled..filled + n].copy_from_slice(&bytes[..n]);
        filled += n;
    }
    Ok(())
}

async fn write_all(writer: &WritableStreamDefaultWriter<Uint8Array>, data: &[u8]) -> Result<()> {
    writer.write_with_chunk(&Uint8Array::from(data)).await?;
    Ok(())
}

async fn handle_tcp(socket: Socket) -> Result<()> {
    let count = CONNECTIONS.fetch_add(1, Ordering::Relaxed) + 1;
    let reader = socket.readable().get_reader()?;
    let writer = socket.writable().get_writer()?;
    let mut request = [0; 4];
    read_exact(&reader, &mut request).await?;
    match &request {
        b"fail" => return Err("connect handler failed".into()),
        b"stat" => write_all(&writer, count.to_string().as_bytes()).await?,
        _ => write_all(&writer, &request).await?,
    }
    writer.release_lock()?;
    reader.release_lock()?;
    socket.close().await?;
    Ok(())
}

async fn handle_udp(socket: Socket) -> Result<()> {
    let reader = socket.readable().get_reader()?;
    let writer = socket.writable().get_writer()?;
    loop {
        let chunk = reader.read().await?;
        if chunk.done() {
            break;
        }
        writer
            .write_with_chunk(&chunk.value().unwrap_or_default())
            .await?;
    }
    Ok(())
}

/// TCP and UDP inbound connections share the `connect` export, dispatched on
/// the socket's protocol as worker-build's shim does.
#[event(connect)]
async fn handle_connect(socket: Socket, _env: Env, _ctx: ExecutionContext) -> Result<()> {
    let protocol = js_sys::Reflect::get(&socket, "protocol")?;
    if protocol.and_then(|p| p.as_string()).as_deref() == Some("udp") {
        handle_udp(socket).await
    } else {
        handle_tcp(socket).await
    }
}

pub async fn handle_socket_failed(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let socket = connect("127.0.0.1:25000")?;

    match socket.opened().await {
        Ok(_) => Err("Socket should have failed to open.".into()),
        Err(e) => Response::builder().ok(format!("{e:?}")),
    }
}

pub async fn handle_socket_read(
    _req: Request,
    _env: Env,
    _data: SomeSharedData,
) -> Result<Response> {
    let socket = connect("127.0.0.1:8080")?;

    socket.opened().await?;

    let writer = socket.writable().get_writer()?;
    write_all(&writer, b"ping").await?;

    let reader = socket.readable().get_reader()?;
    let mut response = [0; 4];
    read_exact(&reader, &mut response).await?;

    assert_eq!(&response, b"ping");

    writer.release_lock()?;
    reader.release_lock()?;
    socket.close().await?;

    socket.closed().await?;

    Response::builder().ok("success")
}
