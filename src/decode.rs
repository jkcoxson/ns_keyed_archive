// Jackson Coxson

use nskeyedarchiver_converter::{Converter, ConverterError};
use plist::Value;

pub fn from_bytes(input: &[u8]) -> Result<Value, ConverterError> {
    let mut decoder = Converter::from_bytes(input).unwrap();
    Ok(flatten_root(&decoder.decode()?))
}

pub fn from_reader<R: std::io::Read + std::io::Seek>(reader: R) -> Result<Value, ConverterError> {
    let mut decoder = Converter::from_reader(reader)?;
    Ok(flatten_root(&decoder.decode()?))
}

pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Value, ConverterError> {
    let mut decoder = Converter::from_file(path)?;
    Ok(flatten_root(&decoder.decode()?))
}

pub fn flatten_root(input: &Value) -> Value {
    println!("Flattening: {input:#?}");
    let mut result = plist::Dictionary::new();

    if let Value::Dictionary(root) = input {
        if let Some(root) = root.get("root") {
            if let Value::Array(items) = root {
                for item in items {
                    if let Value::Dictionary(dict) = item {
                        if let (Some(Value::String(key)), Some(value)) =
                            (dict.get("key"), dict.get("value"))
                        {
                            if let Value::Array(arr) = value {
                                if arr.iter().all(|v| matches!(v, Value::Dictionary(_))) {
                                    let mut nested_map = plist::Dictionary::new();
                                    for v in arr {
                                        if let Value::Dictionary(inner_dict) = v {
                                            if let (
                                                Some(Value::String(inner_key)),
                                                Some(inner_value),
                                            ) = (inner_dict.get("key"), inner_dict.get("value"))
                                            {
                                                nested_map
                                                    .insert(inner_key.clone(), inner_value.clone());
                                            }
                                        }
                                    }
                                    result.insert(key.clone(), Value::Dictionary(nested_map));
                                } else {
                                    result.insert(key.clone(), value.clone());
                                }
                            } else {
                                result.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
            } else {
                return root.to_owned();
            }
        }
    }

    Value::Dictionary(result)
}
