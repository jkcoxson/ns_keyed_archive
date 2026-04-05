//! NSKeyedArchive decoder.
//!
//! Decodes NSKeyedArchive binary plists produced by Apple's Objective-C runtime.
//! Handles all common collection types including private class names used by iOS
//! (e.g. `__NSArrayM`, `__NSArrayI`, `__NSDictionaryM`, `NSMutableSet`, etc.).

use plist::{Dictionary, Value};

use super::ConverterError;

/// Decode an NSKeyedArchive binary plist from a byte slice.
pub fn from_bytes(input: &[u8]) -> Result<Value, ConverterError> {
    let archive = plist::Value::from_reader(std::io::Cursor::new(input))
        .map_err(|e| ConverterError::InvalidObjectEncoding(e.to_string().len() as u64))?;
    decode_archive(archive)
}

/// Decode an NSKeyedArchive from any reader.
pub fn from_reader<R: std::io::Read + std::io::Seek>(reader: R) -> Result<Value, ConverterError> {
    let archive = plist::Value::from_reader(reader)
        .map_err(|e| ConverterError::InvalidObjectEncoding(e.to_string().len() as u64))?;
    decode_archive(archive)
}

/// Decode an NSKeyedArchive from a file.
pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Value, ConverterError> {
    let archive = plist::Value::from_file(path)
        .map_err(|e| ConverterError::InvalidObjectEncoding(e.to_string().len() as u64))?;
    decode_archive(archive)
}

fn decode_archive(archive: Value) -> Result<Value, ConverterError> {
    let dict = archive
        .into_dictionary()
        .ok_or(ConverterError::InvalidObjectEncoding(0))?;

    let objects = dict
        .get("$objects")
        .and_then(|v| v.as_array())
        .ok_or(ConverterError::InvalidObjectEncoding(1))?;

    let top = dict
        .get("$top")
        .and_then(|v| v.as_dictionary())
        .ok_or(ConverterError::InvalidObjectEncoding(2))?;

    let root_uid = top
        .get("root")
        .and_then(uid_to_index)
        .ok_or(ConverterError::InvalidObjectEncoding(3))?;

    decode_object(objects, root_uid)
}

fn uid_to_index(v: &Value) -> Option<usize> {
    match v {
        Value::Uid(u) => Some(u.get() as usize),
        _ => None,
    }
}

fn decode_object(objects: &[Value], idx: usize) -> Result<Value, ConverterError> {
    let obj = objects
        .get(idx)
        .ok_or(ConverterError::InvalidObjectEncoding(idx as u64))?;

    match obj {
        Value::String(s) if s == "$null" => Ok(Value::Dictionary(Dictionary::new())),

        Value::String(_)
        | Value::Integer(_)
        | Value::Real(_)
        | Value::Boolean(_)
        | Value::Date(_)
        | Value::Data(_) => Ok(obj.clone()),

        // Uid indirection — follow the reference
        Value::Uid(u) => decode_object(objects, u.get() as usize),

        Value::Dictionary(d) => {
            // Determine class name for dispatch
            let class_name = d
                .get("$class")
                .and_then(uid_to_index)
                .and_then(|ci| objects.get(ci))
                .and_then(|v| v.as_dictionary())
                .and_then(|cd| cd.get("$classname"))
                .and_then(|v| v.as_string())
                .unwrap_or("");

            // NSString / NSMutableString
            if class_name.contains("String")
                && let Some(s) = d.get("NS.string").and_then(|v| v.as_string())
            {
                return Ok(Value::String(s.to_string()));
            }

            // NSData / NSMutableData
            if class_name.contains("Data")
                && let Some(bytes) = d.get("NS.data").and_then(|v| v.as_data())
            {
                return Ok(Value::Data(bytes.to_vec()));
            }

            // NSDictionary variants (also matches __NSDictionaryI, __NSDictionaryM, etc.)
            if class_name.contains("Dictionary") {
                let keys = d.get("NS.keys").and_then(|v| v.as_array());
                let vals = d.get("NS.objects").and_then(|v| v.as_array());
                if let (Some(keys), Some(vals)) = (keys, vals) {
                    let mut result = Dictionary::new();
                    for (k, v) in keys.iter().zip(vals.iter()) {
                        if let (Some(ki), Some(vi)) = (uid_to_index(k), uid_to_index(v)) {
                            let key_val = decode_object(objects, ki)?;
                            let val_val = decode_object(objects, vi)?;
                            let key_str = match &key_val {
                                Value::String(s) => s.clone(),
                                Value::Integer(i) => i.to_string(),
                                Value::Real(f) => f.to_string(),
                                _ => format!("{key_val:?}"),
                            };
                            result.insert(key_str, val_val);
                        }
                    }
                    return Ok(Value::Dictionary(result));
                }
                return Ok(Value::Dictionary(Dictionary::new()));
            }

            // NSArray / NSMutableArray / NSSet / NSMutableSet and their private variants.
            // All use NS.objects (no NS.keys), treated as ordered arrays.
            if class_name.contains("Array") || class_name.contains("Set") {
                let items = d.get("NS.objects").and_then(|v| v.as_array());
                if let Some(items) = items {
                    let mut result = Vec::new();
                    for item in items {
                        if let Some(i) = uid_to_index(item) {
                            result.push(decode_object(objects, i)?);
                        }
                    }
                    return Ok(Value::Array(result));
                }
                return Ok(Value::Array(Vec::new()));
            }

            Ok(obj.clone())
        }

        // Arrays in the $objects pool are unusual but handle gracefully
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                if let Some(i) = uid_to_index(item) {
                    result.push(decode_object(objects, i)?);
                } else {
                    result.push(item.clone());
                }
            }
            Ok(Value::Array(result))
        }

        // plist::Value is non-exhaustive; return unknown variants as-is
        _ => Ok(obj.clone()),
    }
}

pub fn flatten_root(input: &Value) -> Value {
    input.clone()
}
