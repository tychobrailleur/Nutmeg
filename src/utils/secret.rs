pub fn deobfuscate(encoded: String) -> String {
    if let Some(hex_str) = encoded.strip_prefix("OBF:") {
        let key = b"nutmeg";

        if hex_str.len() % 2 != 0 {
            return encoded;
        }

        let mut bytes = Vec::with_capacity(hex_str.len() / 2);
        for i in (0..hex_str.len()).step_by(2) {
            if let Ok(b) = u8::from_str_radix(&hex_str[i..i + 2], 16) {
                bytes.push(b);
            } else {
                return encoded;
            }
        }

        let mut result = Vec::with_capacity(bytes.len());
        for (i, byte) in bytes.iter().enumerate() {
            result.push(byte ^ key[i % key.len()]);
        }

        String::from_utf8(result).unwrap_or(encoded)
    } else {
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deobfuscate() {
        assert_eq!(
            deobfuscate("local_dev_secret".to_string()),
            "local_dev_secret"
        );
        assert_eq!(deobfuscate("OBF:1a100719".to_string()), "test");
        assert_eq!(deobfuscate("OBF:invalid".to_string()), "OBF:invalid");
    }
}
