/// Jackson Coxson
use nskeyedarchiver_converter::ConverterError;
use plist::{Dictionary, Uid, Value};

const ARCHIVER: &str = "NSKeyedArchiver";
const ARCHIVER_VERSION: u64 = 100000;

const ARCHIVER_KEY_NAME: &str = "$archiver";
const TOP_KEY_NAME: &str = "$top";
const OBJECTS_KEY_NAME: &str = "$objects";
const VERSION_KEY_NAME: &str = "$version";
const NULL_OBJECT_REFERENCE_NAME: &str = "$null";

/// Encodes a plist Value into NSKeyedArchiver format.
///
/// If successful, returns a plist::Value representing an NSKeyedArchiver encoded plist.
pub fn encode(value: Value) -> Result<Value, ConverterError> {
    // Initialize the objects array with $null as the first element (index 0)
    let mut objects = vec![Value::String(NULL_OBJECT_REFERENCE_NAME.to_string())];

    // Create the top-level structure
    let mut root_dict = Dictionary::new();

    // Add required headers
    root_dict.insert(
        ARCHIVER_KEY_NAME.to_string(),
        Value::String(ARCHIVER.to_string()),
    );
    root_dict.insert(
        VERSION_KEY_NAME.to_string(),
        Value::Integer(ARCHIVER_VERSION.into()),
    );

    // Process the root value - it might be any kind of value, but we wrap it in "root"
    let mut top_dict = Dictionary::new();
    let uid = encode_object(value, &mut objects)?;
    top_dict.insert("root".to_string(), Value::Uid(uid));

    // Add top and objects to the root dictionary
    root_dict.insert(TOP_KEY_NAME.to_string(), Value::Dictionary(top_dict));
    root_dict.insert(OBJECTS_KEY_NAME.to_string(), Value::Array(objects));

    Ok(Value::Dictionary(root_dict))
}

/// Encodes a single object and returns its UID
fn encode_object(value: Value, objects: &mut Vec<Value>) -> Result<Uid, ConverterError> {
    match value {
        Value::Dictionary(dict) => encode_dictionary(dict, objects),
        Value::Array(array) => encode_array(array, objects),
        Value::Boolean(b) => {
            // Add the boolean to objects array
            objects.push(Value::Boolean(b));
            Ok(Uid::new(objects.len() as u64 - 1))
        }
        Value::Real(r) => {
            objects.push(Value::Real(r));
            Ok(Uid::new(objects.len() as u64 - 1))
        }
        Value::Integer(i) => {
            objects.push(Value::Integer(i));
            Ok(Uid::new(objects.len() as u64 - 1))
        }
        Value::String(s) => {
            if s == NULL_OBJECT_REFERENCE_NAME {
                return Ok(Uid::new(0)); // Reference to $null
            }
            objects.push(Value::String(s));
            Ok(Uid::new(objects.len() as u64 - 1))
        }
        Value::Date(d) => {
            objects.push(Value::Date(d));
            Ok(Uid::new(objects.len() as u64 - 1))
        }
        Value::Data(d) => {
            objects.push(Value::Data(d));
            Ok(Uid::new(objects.len() as u64 - 1))
        }
        Value::Uid(_) => {
            // UIDs should be handled differently in the context they appear
            Err(ConverterError::InvalidObjectEncoding(0))
        }
        _ => unimplemented!(),
    }
}

/// Encodes a dictionary as an NSDictionary
fn encode_dictionary(dict: Dictionary, objects: &mut Vec<Value>) -> Result<Uid, ConverterError> {
    // Create arrays for keys and values
    let mut key_uids = Vec::new();
    let mut value_uids = Vec::new();

    // Process all key-value pairs
    for (key, value) in dict {
        // First encode the key (usually a string)
        let key_uid = encode_object(Value::String(key), objects)?;
        key_uids.push(Value::Uid(key_uid));

        // Then encode the value
        let value_uid = encode_object(value, objects)?;
        value_uids.push(Value::Uid(value_uid));
    }

    // Get or create the NSDictionary class reference
    let class_uid = create_class_reference("NSDictionary", objects)?;

    // Build the NSKeyedArchiver structure for a dictionary
    let mut dict_structure = Dictionary::new();
    dict_structure.insert("$class".to_string(), Value::Uid(class_uid));
    dict_structure.insert("NS.keys".to_string(), Value::Array(key_uids));
    dict_structure.insert("NS.objects".to_string(), Value::Array(value_uids));

    // Add the structured dictionary to objects and return its UID
    objects.push(Value::Dictionary(dict_structure));
    Ok(Uid::new(objects.len() as u64 - 1))
}

/// Encodes an array as an NSArray
fn encode_array(array: Vec<Value>, objects: &mut Vec<Value>) -> Result<Uid, ConverterError> {
    // Encode array elements
    let mut object_uids = Vec::new();

    for item in array {
        let item_uid = encode_object(item, objects)?;
        object_uids.push(Value::Uid(item_uid));
    }

    // Create class reference for NSArray
    let class_uid = create_class_reference("NSArray", objects)?;

    // Create array structure
    let mut array_structure = Dictionary::new();
    array_structure.insert("$class".to_string(), Value::Uid(class_uid));
    array_structure.insert("NS.objects".to_string(), Value::Array(object_uids));

    // Add to objects array
    objects.push(Value::Dictionary(array_structure));
    Ok(Uid::new(objects.len() as u64 - 1))
}

/// Creates a class reference dictionary and returns its UID
fn create_class_reference(
    class_name: &str,
    objects: &mut Vec<Value>,
) -> Result<Uid, ConverterError> {
    // Check if we already have this class reference
    for (i, obj) in objects.iter().enumerate() {
        if let Some(dict) = obj.as_dictionary() {
            if let Some(classes) = dict.get("$classes").and_then(|c| c.as_array()) {
                if let Some(name) = classes.first().and_then(|n| n.as_string()) {
                    if name == class_name {
                        return Ok(Uid::new(i as u64));
                    }
                }
            }
        }
    }

    // Create class definition
    let mut class_dict = Dictionary::new();
    class_dict.insert(
        "$classes".to_string(),
        Value::Array(vec![
            Value::String(class_name.to_string()),
            Value::String("NSObject".to_string()),
        ]),
    );
    class_dict.insert(
        "$classname".to_string(),
        Value::String(class_name.to_string()),
    );

    // Add to objects array
    objects.push(Value::Dictionary(class_dict));
    Ok(Uid::new(objects.len() as u64 - 1))
}

/// Encodes a Value to NSKeyedArchiver format and writes it to a file
pub fn encode_to_file<P: AsRef<std::path::Path>>(
    value: Value,
    path: P,
) -> Result<(), ConverterError> {
    let encoded = encode(value)?;
    encoded.to_file_binary(path)?;
    Ok(())
}

/// Encodes a Value to NSKeyedArchiver format and returns it as bytes
pub fn encode_to_bytes(value: Value) -> Result<Vec<u8>, ConverterError> {
    let encoded = encode(value)?;

    let buf = Vec::new();
    let mut writer = std::io::BufWriter::new(buf);
    plist::to_writer_binary(&mut writer, &encoded).unwrap();

    Ok(writer.into_inner().unwrap())
}

/// Encodes a Value to NSKeyedArchiver format and writes it to a writer
pub fn encode_to_writer<W: std::io::Write>(value: Value, writer: W) -> Result<(), ConverterError> {
    let encoded = encode(value)?;
    plist::to_writer_binary(writer, &encoded)?;
    Ok(())
}
