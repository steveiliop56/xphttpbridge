use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use xplm::data::borrowed::DataRef;
use xplm::data::{
    ArrayRead, ArrayReadWrite, DataRead, DataReadWrite, DataType, ReadOnly, ReadWrite,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RefValue {
    Bool(bool),
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RefValues {
    SF32(Vec<f32>),
    SU32(Vec<u32>),
    SI32(Vec<i32>),
    SU8(Vec<u8>),
    SI8(Vec<i8>),
}

#[derive(Serialize, Clone)]
pub struct DataRefInfo {
    pub ref_name: String,
    pub ref_type: String,
    pub writable: bool,
    pub value_type: String,
    pub value_description: String,
}

pub fn get_ref_value(ref_name: &str) -> Option<RefValue> {
    if let Ok(r) = DataRef::<bool>::find(ref_name) {
        return Some(RefValue::Bool(DataRead::<bool>::get(&r)));
    }
    if let Ok(r) = DataRef::<f32>::find(ref_name) {
        return Some(RefValue::F32(DataRead::<f32>::get(&r)));
    }
    if let Ok(r) = DataRef::<f64>::find(ref_name) {
        return Some(RefValue::F64(DataRead::<f64>::get(&r)));
    }
    if let Ok(r) = DataRef::<i8>::find(ref_name) {
        return Some(RefValue::I8(DataRead::<i8>::get(&r)));
    }
    if let Ok(r) = DataRef::<i16>::find(ref_name) {
        return Some(RefValue::I16(DataRead::<i16>::get(&r)));
    }
    if let Ok(r) = DataRef::<i32>::find(ref_name) {
        return Some(RefValue::I32(DataRead::<i32>::get(&r)));
    }
    if let Ok(r) = DataRef::<u8>::find(ref_name) {
        return Some(RefValue::U8(DataRead::<u8>::get(&r)));
    }
    if let Ok(r) = DataRef::<u16>::find(ref_name) {
        return Some(RefValue::U16(DataRead::<u16>::get(&r)));
    }
    if let Ok(r) = DataRef::<u32>::find(ref_name) {
        return Some(RefValue::U32(DataRead::<u32>::get(&r)));
    }
    None
}

pub fn set_ref_value(ref_name: &str, ref_value: RefValue) -> bool {
    match ref_value {
        RefValue::Bool(v) => set_ref_value_callback(ref_name, v),
        RefValue::F32(v) => set_ref_value_callback(ref_name, v),
        RefValue::F64(v) => set_ref_value_callback(ref_name, v),
        RefValue::I8(v) => set_ref_value_callback(ref_name, v),
        RefValue::I16(v) => set_ref_value_callback(ref_name, v),
        RefValue::I32(v) => set_ref_value_callback(ref_name, v),
        RefValue::U8(v) => set_ref_value_callback(ref_name, v),
        RefValue::U16(v) => set_ref_value_callback(ref_name, v),
        RefValue::U32(v) => set_ref_value_callback(ref_name, v),
    }
}

fn set_ref_value_callback<T: DataType>(ref_name: &str, ref_value: T) -> bool
where
    DataRef<T, ReadOnly>: DataRead<T>,
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
pub fn get_ref_values(ref_name: &str) -> Option<RefValues> {
    if let Ok(r) = DataRef::<[f32]>::find(ref_name) {
        return Some(RefValues::SF32(r.as_vec()));
    }
    if let Ok(r) = DataRef::<[u32]>::find(ref_name) {
        return Some(RefValues::SU32(r.as_vec()));
    }
    if let Ok(r) = DataRef::<[i32]>::find(ref_name) {
        return Some(RefValues::SI32(r.as_vec()));
    }
    if let Ok(r) = DataRef::<[u8]>::find(ref_name) {
        return Some(RefValues::SU8(r.as_vec()));
    }
    if let Ok(r) = DataRef::<[i8]>::find(ref_name) {
        return Some(RefValues::SI8(r.as_vec()));
    }
    None
}

pub fn set_ref_values(ref_name: &str, ref_values: RefValues) -> bool {
    match ref_values {
        RefValues::SF32(v) => {
            if let Ok(r) = DataRef::<[f32]>::find(ref_name) {
                if let Ok(mut rw) = r.writeable() {
                    rw.set(&v);
                    return true;
                }
            }
            false
        }
        RefValues::SU32(v) => {
            if let Ok(r) = DataRef::<[u32]>::find(ref_name) {
                if let Ok(mut rw) = r.writeable() {
                    rw.set(&v);
                    return true;
                }
            }
            false
        }
        RefValues::SI32(v) => {
            if let Ok(r) = DataRef::<[i32]>::find(ref_name) {
                if let Ok(mut rw) = r.writeable() {
                    rw.set(&v);
                    return true;
                }
            }
            false
        }
        RefValues::SU8(v) => {
            if let Ok(r) = DataRef::<[u8]>::find(ref_name) {
                if let Ok(mut rw) = r.writeable() {
                    rw.set(&v);
                    return true;
                }
            }
            false
        }
        RefValues::SI8(v) => {
            if let Ok(r) = DataRef::<[i8]>::find(ref_name) {
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
    if parts.len() < 5 {
        return None;
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
