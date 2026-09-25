# UDP Socket Example

This example handles inbound UDP datagrams using `#[event(connect(udp))]`.

Each `UdpSocket::recv()` result is one complete datagram, and each
`UdpSocket::send()` call sends one complete datagram.

## Usage

```bash
npm install
npm run dev
```
