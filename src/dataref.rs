use serde::Serialize;
use xplm::data::DataRead;
use xplm::data::borrowed::DataRef;

#[derive(Serialize)]
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
