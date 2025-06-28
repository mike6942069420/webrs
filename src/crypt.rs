use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;

pub fn generate_nonce_base64(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    general_purpose::STANDARD.encode(bytes)
}
