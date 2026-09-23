// OrbitOS — Minimalist Protobuf (proto3) wire parser and encoder

use std::collections::HashMap;

//=========================================#
//            PROTO TYPES & VALUES         #
//=========================================#

#[derive(Debug, Clone, PartialEq)]
pub enum WireValue {
    Varint(u64),
    Bytes(Vec<u8>),
    Fixed64([u8; 8]),
    Fixed32([u8; 4]),
}

pub type ProtoFields = HashMap<u32, Vec<WireValue>>;

// Parse raw protobuf binary into tag map
pub fn parse_proto(b: &[u8]) -> ProtoFields {
    let mut fields: ProtoFields = HashMap::new();
    let mut i = 0;

    while i < b.len() {
        let mut shift = 0u32;
        let mut tag_wire = 0u64;

        // Read tag + wire varint
        loop {
            if i >= b.len() {
                return fields;
            }
            let byte = b[i];
            tag_wire |= ((byte & 0x7f) as u64) << shift;
            i += 1;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
            if shift >= 64 {
                return fields;
            }
        }

        let tag = (tag_wire >> 3) as u32;
        let wire = (tag_wire & 7) as u8;

        match wire {
            0 => {
                // Varint
                let mut val = 0u64;
                let mut shift = 0u32;
                loop {
                    if i >= b.len() {
                        return fields;
                    }
                    let byte = b[i];
                    val |= ((byte & 0x7f) as u64) << shift;
                    i += 1;
                    if (byte & 0x80) == 0 {
                        break;
                    }
                    shift += 7;
                    if shift >= 64 {
                        return fields;
                    }
                }
                fields.entry(tag).or_default().push(WireValue::Varint(val));
            }
            2 => {
                // Length-delimited (bytes / string / submessage)
                let mut len = 0usize;
                let mut shift = 0u32;
                loop {
                    if i >= b.len() {
                        return fields;
                    }
                    let byte = b[i];
                    len |= ((byte & 0x7f) as usize) << shift;
                    i += 1;
                    if (byte & 0x80) == 0 {
                        break;
                    }
                    shift += 7;
                    if shift >= 64 {
                        return fields;
                    }
                }
                if i + len > b.len() {
                    return fields;
                }
                let sub = b[i..i + len].to_vec();
                i += len;
                fields.entry(tag).or_default().push(WireValue::Bytes(sub));
            }
            1 => {
                // 64-bit fixed
                if i + 8 > b.len() {
                    return fields;
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&b[i..i + 8]);
                i += 8;
                fields.entry(tag).or_default().push(WireValue::Fixed64(arr));
            }
            5 => {
                // 32-bit fixed
                if i + 4 > b.len() {
                    return fields;
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&b[i..i + 4]);
                i += 4;
                fields.entry(tag).or_default().push(WireValue::Fixed32(arr));
            }
            _ => {
                // Unknown wire type, cannot proceed safely
                break;
            }
        }
    }

    fields
}

//=========================================#
//            ENCODING HELPERS             #
//=========================================#

pub fn encode_varint(mut val: u64) -> Vec<u8> {
    let mut res = Vec::new();
    loop {
        let b = (val & 0x7f) as u8;
        val >>= 7;
        if val != 0 {
            res.push(b | 0x80);
        } else {
            res.push(b);
            break;
        }
    }
    res
}

pub fn encode_field_varint(tag: u32, val: u64) -> Vec<u8> {
    let header = encode_varint(((tag as u64) << 3) | 0);
    let mut res = header;
    res.extend(encode_varint(val));
    res
}

pub fn encode_field_bytes(tag: u32, data: &[u8]) -> Vec<u8> {
    let header = encode_varint(((tag as u64) << 3) | 2);
    let mut res = header;
    res.extend(encode_varint(data.len() as u64));
    res.extend_from_slice(data);
    res
}

pub fn make_timestamp(sec: u64, nano: u64) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(encode_field_varint(1, sec));
    out.extend(encode_field_varint(2, nano));
    out
}

pub fn make_ws_info(uri: &str, repo: &str, repo_url: &str, branch: &str) -> Vec<u8> {
    let mut repo_msg = Vec::new();
    repo_msg.extend(encode_field_bytes(1, repo.as_bytes()));
    repo_msg.extend(encode_field_bytes(2, repo_url.as_bytes()));

    let mut out = Vec::new();
    out.extend(encode_field_bytes(1, uri.as_bytes()));
    out.extend(encode_field_bytes(2, uri.as_bytes()));
    out.extend(encode_field_bytes(3, &repo_msg));
    out.extend(encode_field_bytes(4, branch.as_bytes()));
    out
}

pub trait ProtoFieldsExt {
    fn get_first_bytes(&self, tag: u32) -> Option<&[u8]>;
    fn get_first_string(&self, tag: u32) -> Option<String>;
    fn get_first_varint(&self, tag: u32) -> Option<u64>;
}

impl ProtoFieldsExt for ProtoFields {
    fn get_first_bytes(&self, tag: u32) -> Option<&[u8]> {
        self.get(&tag)?.iter().find_map(|v| match v {
            WireValue::Bytes(b) => Some(b.as_slice()),
            _ => None,
        })
    }

    fn get_first_string(&self, tag: u32) -> Option<String> {
        let bytes = self.get_first_bytes(tag)?;
        Some(String::from_utf8_lossy(bytes).to_string())
    }

    fn get_first_varint(&self, tag: u32) -> Option<u64> {
        self.get(&tag)?.iter().find_map(|v| match v {
            WireValue::Varint(val) => Some(*val),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_roundtrip() {
        for val in [0u64, 1, 127, 128, 300, 16384, u32::MAX as u64, u64::MAX] {
            let enc = encode_varint(val);
            let fields = parse_proto(&encode_field_varint(1, val));
            assert_eq!(fields.get_first_varint(1), Some(val));
        }
    }

    #[test]
    fn test_bytes_field_roundtrip() {
        let msg = b"Hello world protobuf";
        let enc = encode_field_bytes(5, msg);
        let parsed = parse_proto(&enc);
        assert_eq!(parsed.get_first_bytes(5), Some(msg.as_slice()));
        assert_eq!(parsed.get_first_string(5), Some("Hello world protobuf".to_string()));
    }

    #[test]
    fn test_make_timestamp() {
        let ts = make_timestamp(1700000000, 123456);
        let parsed = parse_proto(&ts);
        assert_eq!(parsed.get_first_varint(1), Some(1700000000));
        assert_eq!(parsed.get_first_varint(2), Some(123456));
    }

    #[test]
    fn test_make_ws_info() {
        let ws = make_ws_info("file:///etc/nixos", "Orbit-Nix/orbit-config", "git@github.com:Orbit-Nix/orbit-config.git", "main");
        let parsed = parse_proto(&ws);
        assert_eq!(parsed.get_first_string(1), Some("file:///etc/nixos".to_string()));
        assert_eq!(parsed.get_first_string(2), Some("file:///etc/nixos".to_string()));
        assert_eq!(parsed.get_first_string(4), Some("main".to_string()));

        let repo_bytes = parsed.get_first_bytes(3).expect("repo_msg");
        let repo_parsed = parse_proto(repo_bytes);
        assert_eq!(repo_parsed.get_first_string(1), Some("Orbit-Nix/orbit-config".to_string()));
        assert_eq!(repo_parsed.get_first_string(2), Some("git@github.com:Orbit-Nix/orbit-config.git".to_string()));
    }
}
