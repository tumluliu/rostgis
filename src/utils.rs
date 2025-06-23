/// Utility functions for RostGIS extension
use std::error::Error;
use std::fmt;

/// Custom error type for RostGIS operations
#[derive(Debug)]
pub struct RostGisError {
    pub message: String,
}

impl fmt::Display for RostGisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RostGIS Error: {}", self.message)
    }
}

impl Error for RostGisError {}

impl RostGisError {
    pub fn new(message: &str) -> Self {
        RostGisError {
            message: message.to_string(),
        }
    }
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let hex = if hex.starts_with("0x") {
        &hex[2..]
    } else {
        hex
    };

    if hex.len() % 2 != 0 {
        return Err(RostGisError::new("Invalid hex string length").into());
    }

    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| RostGisError::new("Invalid hex character"))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Validate SRID value
pub fn validate_srid(srid: i32) -> Result<i32, RostGisError> {
    if srid < -1 {
        Err(RostGisError::new("Invalid SRID: must be >= -1"))
    } else {
        Ok(srid)
    }
}

/// Common SRID constants
pub mod srid {
    pub const UNKNOWN: i32 = 0;
    pub const WGS84: i32 = 4326;
    pub const WEB_MERCATOR: i32 = 3857;
    pub const NAD83: i32 = 4269;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_conversion() {
        let hex = "deadbeef";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);

        let hex_back = bytes_to_hex(&bytes);
        assert_eq!(hex_back, hex);
    }

    #[test]
    fn test_srid_validation() {
        assert!(validate_srid(4326).is_ok());
        assert!(validate_srid(0).is_ok());
        assert!(validate_srid(-1).is_ok());
        assert!(validate_srid(-2).is_err());
    }

    #[test]
    fn test_hex_with_prefix() {
        let hex = "0xdeadbeef";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }
}
