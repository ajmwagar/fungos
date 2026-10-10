# Private torrent-tool qualification

On 2026-10-10, the ignored `loopback_download` integration test transferred
8,388,608 synthetic bytes from one Transmission 4.1.1 client to a second,
initially empty client. The received SHA-256 was
`60606d40d8865c0642feb0ef74d2d8793d27b52df9db9d1a60d3c7a1b3bdaa94`.
The actual successful run took 23.49 seconds, against test source `7548892`.

Both clients ran inside a fresh network namespace containing only `lo`, with
DHT, PEX, LPD and port mapping disabled. A private `10.254.0.1/32` alias on that
namespace's loopback device carried the peer connection; RPC and the tiny
test tracker used 127.0.0.1. There was no external interface, public tracker,
web seed, payload-serving tracker or upload/publication step. The tracker only
announced the two local peers; matching payload bytes came from the seed client.
Owned daemons were stopped and the successful synthetic fixture cleaned up.

Earlier 127/8 peer-address trials failed: Transmission deliberately rejects
tracker-learned loopback addresses. Its upstream
[peer address validation](https://github.com/transmission/transmission/blob/4.1.1/libtransmission/net.cc)
explains the distinction. The namespace-local private alias fixes the test
without altering host networking or weakening Transmission validation.

Reproduction requires the Transmission CLI/daemon tools and root permission to
create a fresh namespace. Build the test with `cargo test --locked --manifest-path
release/Cargo.toml --test loopback_download --no-run`, then run the emitted test
executable with `sudo unshare --net -- TEST_EXECUTABLE --ignored --nocapture`.
It rejects the host namespace and any namespace containing non-loopback
interfaces before changing any routes or addresses. Failures retain synthetic
logs; they are not acceptance evidence.

This qualifies an actual two-client transfer, not public swarm availability,
release authority signatures, a final ISO or complete corresponding-source
coverage. Those remain separate release gates; see [SOURCE-CLOSURE.md](SOURCE-CLOSURE.md).
