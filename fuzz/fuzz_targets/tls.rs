//! tls.rs — fuzzes the TLS client with server bytes, using the browser's crypto provider with the in-tree X25519MLKEM768 key exchange first and a fixed random source so the ClientHello is reproducible.
#![no_main]

#[allow(dead_code)]
#[path = "../../rust/http/src/x25519_mlkem768.rs"]
mod x25519_mlkem768;

use std::io::Cursor;
use std::sync::{Arc, OnceLock};

use rustls::client::ClientConnection;
use rustls::crypto::{GetRandomFailed, SecureRandom};
use rustls::{ClientConfig, RootCertStore};

#[derive(Debug)]
struct FixedRandom;

impl SecureRandom for FixedRandom {
    fn fill(&self, buf: &mut [u8]) -> Result<(), GetRandomFailed> {
        buf.fill(0x5a);
        Ok(())
    }
}

fn config() -> Arc<ClientConfig> {
    static CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let mut provider = rustls::crypto::ring::default_provider();
            provider.kx_groups.insert(0, x25519_mlkem768::X25519MLKEM768);
            provider.secure_random = &FixedRandom;
            let mut config = ClientConfig::builder_with_provider(Arc::new(provider))
                .with_safe_default_protocol_versions()
                .expect("default protocol versions")
                .with_root_certificates(RootCertStore::empty())
                .with_no_client_auth();
            config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            Arc::new(config)
        })
        .clone()
}

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Ok(mut connection) = ClientConnection::new(config(), "example.com".try_into().expect("server name")) else {
        return;
    };
    let mut sink = Vec::new();
    let _ = connection.write_tls(&mut sink);
    let mut cursor = Cursor::new(data);
    while (cursor.position() as usize) < data.len() {
        match connection.read_tls(&mut cursor) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        if connection.process_new_packets().is_err() {
            break;
        }
        sink.clear();
        let _ = connection.write_tls(&mut sink);
    }
});
