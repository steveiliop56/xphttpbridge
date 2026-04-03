use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, DataRead, DataReadWrite, DataType, ReadWrite};
use xplm::debugln;

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RefValue {
    Int(i32),
    Float(f32),
    Double(f64),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RefValues {
    Ints(Vec<i32>),
    Floats(Vec<f32>),
    Bytes(Vec<u32>),
}

#[derive(Serialize, Clone, Debug)]
pub struct DataRefInfo {
    pub ref_name: String,
    pub ref_type: String,
    pub writable: bool,
    pub value_type: String,
    pub value_description: String,
}

pub fn get_ref_value(ref_name: &str, ref_type: RefValue) -> Option<RefValue> {
    match ref_type {
        RefValue::Float(_) => {
            if let Ok(r) = DataRef::<f32>::find(ref_name) {
                return Some(RefValue::Float(DataRead::<f32>::get(&r)));
            }
        }
        RefValue::Double(_) => {
            if let Ok(r) = DataRef::<f64>::find(ref_name) {
                return Some(RefValue::Double(DataRead::<f64>::get(&r)));
            }
        }
        RefValue::Int(_) => {
            if let Ok(r) = DataRef::<i32>::find(ref_name) {
                return Some(RefValue::Int(DataRead::<i32>::get(&r)));
            }
        }
    }
    None
}

pub fn set_ref_value(ref_name: &str, ref_value: RefValue) -> bool {
    match ref_value {
        RefValue::Float(v) => set_ref_value_callback(ref_name, v),
        RefValue::Double(v) => set_ref_value_callback(ref_name, v),
        RefValue::Int(v) => set_ref_value_callback(ref_name, v),
    }
}

fn set_ref_value_callback<T: DataType>(ref_name: &str, ref_value: T) -> bool
where
    DataRef<T, ReadWrite>: DataReadWrite<T>,
{
    if let Ok(r) = DataRef::<T>::find(ref_name) {
        if let Ok(mut rw) = r.writeable() {
            DataReadWrite::<T>::set(&mut rw, ref_value);
            return true;
        }
    }
    false
}
pub fn get_ref_values(ref_name: &str, ref_type: RefValues) -> Option<RefValues> {
    match ref_type {
        RefValues::Floats(_) => {
            if let Ok(r) = DataRef::<[f32]>::find(ref_name) {
                return Some(RefValues::Floats(r.as_vec()));
            }
        }
        RefValues::Bytes(_) => {
            if let Ok(r) = DataRef::<[u32]>::find(ref_name) {
                return Some(RefValues::Bytes(r.as_vec()));
            }
        }
        RefValues::Ints(_) => {
            if let Ok(r) = DataRef::<[i32]>::find(ref_name) {
                return Some(RefValues::Ints(r.as_vec()));
            }
        }
    }
    None
}

pub fn set_ref_values(ref_name: &str, ref_values: RefValues) -> bool {
    match ref_values {
        RefValues::Floats(v) => {
            if let Ok(r) = DataRef::<[f32]>::find(ref_name) {
                if let Ok(mut rw) = r.writeable() {
                    rw.set(&v);
                    return true;
                }
            }
            false
        }
        RefValues::Ints(v) => {
            if let Ok(r) = DataRef::<[i32]>::find(ref_name) {
                if let Ok(mut rw) = r.writeable() {
                    rw.set(&v);
                    return true;
                }
            }
            false
        }
        RefValues::Bytes(v) => {
            if let Ok(r) = DataRef::<[u32]>::find(ref_name) {
                if let Ok(mut rw) = r.writeable() {
                    rw.set(&v);
                    return true;
                }
            }
            false
        }
    }
}

pub fn load_and_parse_datarefs(data_ref_path: &str) -> Vec<DataRefInfo> {
    let file_open_res = File::open(data_ref_path);
    let mut file = match file_open_res {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut contents = String::new();
    let read_res = file.read_to_string(&mut contents);
    match read_res {
        Ok(_) => (),
        Err(_) => return Vec::new(),
    };
    let mut data_refs: Vec<DataRefInfo> = Vec::new();
    let lines = contents.lines();
    for (index, line) in lines.enumerate() {
        // Skip the first lines (gen info and empty line)
        if index == 0 || index == 1 {
            continue;
        }
        if let Some(info) = parse_dataref_line(line) {
            data_refs.push(info);
        }
    }
    data_refs
}

fn parse_dataref_line(line: &str) -> Option<DataRefInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        debugln!("XPHTTPBridge: Invalid dataref line: {}", line);
        return None;
    }
    if parts.len() == 3 {
        return Some(DataRefInfo {
            ref_name: parts[0].to_string(),
            ref_type: parts[1].to_string(),
            writable: parts[2].to_string() == "y",
            value_type: String::new(),
            value_description: String::new(),
        });
    }
    let mut final_parts: Vec<&str> = Vec::new();
    for i in 0..4 {
        final_parts.push(parts[i]);
    }
    let value_description = parts[4..].join(" ");
    final_parts.push(&value_description);
    Some(DataRefInfo {
        ref_name: final_parts[0].to_string(),
        ref_type: final_parts[1].to_string(),
        writable: final_parts[2].to_string() == "y",
        value_type: final_parts[3].to_string(),
        value_description: final_parts[4].to_string(),
    })
}

pub fn compile_dataref_type_map(datarefs: Vec<DataRefInfo>) -> ahash::AHashMap<String, String> {
    let mut map = ahash::AHashMap::<String, String>::new();
    let re = Regex::new(r"\[.*\]").unwrap();
    for dataref in datarefs {
        if dataref.ref_type.contains("[") {
            let data_ref_type = re.replace_all(&dataref.ref_type, "[]").to_string();
            map.insert(dataref.ref_name.clone(), data_ref_type);
            continue;
        }
        map.insert(dataref.ref_name.clone(), dataref.ref_type.clone());
    }
    map
}

pub fn map_string_to_ref_value_type(type_str: &str) -> Option<RefValue> {
    match type_str {
        "float" => Some(RefValue::Float(0.0)),
        "double" => Some(RefValue::Double(0.0)),
        "int" => Some(RefValue::Int(0)),
        _ => None,
    }
}

pub fn map_string_to_ref_values_type(type_str: &str) -> Option<RefValues> {
    match type_str {
        "float[]" => Some(RefValues::Floats(Vec::new())),
        "int[]" => Some(RefValues::Ints(Vec::new())),
        "byte[]" => Some(RefValues::Bytes(Vec::new())),
        _ => None,
    }
}
