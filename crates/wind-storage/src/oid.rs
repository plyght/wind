use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Oid([u8; 32]);

impl Oid {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(bytes);
            Some(Self(arr))
        } else {
            None
        }
    }

    pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
        if hex.len() != 64 {
            anyhow::bail!("Invalid hex length: expected 64, got {}", hex.len());
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|_| anyhow::anyhow!("Invalid hex character"))?;
        }
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn hash_bytes(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(*hash.as_bytes())
    }

    pub fn fanout_path(&self) -> (String, String) {
        let hex = self.to_hex();
        (hex[..2].to_string(), hex[2..].to_string())
    }
}

impl fmt::Debug for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Oid({})", &self.to_hex()[..16])
    }
}

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let data = b"hello world";
        let oid = Oid::hash_bytes(data);
        assert_eq!(oid.as_bytes().len(), 32);
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = b"test data";
        let oid = Oid::hash_bytes(data);
        let hex = oid.to_hex();
        let oid2 = Oid::from_hex(&hex).expect("Failed to parse hex");
        assert_eq!(oid, oid2);
    }

    #[test]
    fn test_fanout() {
        let oid = Oid::hash_bytes(b"test");
        let (dir, file) = oid.fanout_path();
        assert_eq!(dir.len(), 2);
        assert_eq!(file.len(), 62);
    }
}
