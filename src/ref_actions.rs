use serde::{Deserialize, Serialize};
use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, DataReadWrite, DataType, ReadOnly, ReadWrite};

pub struct RefActions {}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
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

impl RefActions {
    pub fn new() -> Self {
        Self {}
    }

    pub fn has_ref(&self, path: &str) -> bool {
        // we don't care about the value, just whether the ref exists
        let ref_res = DataRef::<bool, ReadOnly>::find(path);
        match ref_res {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn is_writeable(&self, path: &str) -> bool {
        match self.get_ref(path) {
            Some(DataValue::F32(_)) => DataRef::<f32>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::F64(_)) => DataRef::<f64>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::I32(_)) => DataRef::<i32>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::I16(_)) => DataRef::<i16>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::U32(_)) => DataRef::<u32>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::U16(_)) => DataRef::<u16>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::U8(_)) => DataRef::<u8>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::I8(_)) => DataRef::<i8>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            Some(DataValue::Bool(_)) => DataRef::<bool>::find(path)
                .map(|r| r.writeable().is_ok())
                .unwrap_or(false),
            None => false,
        }
    }

    pub fn get_ref(&self, path: &str) -> Option<DataValue> {
        if let Ok(r) = DataRef::<f32>::find(path) {
            return Some(DataValue::F32(DataRead::<f32>::get(&r)));
        }
        if let Ok(r) = DataRef::<f64>::find(path) {
            return Some(DataValue::F64(DataRead::<f64>::get(&r)));
        }
        if let Ok(r) = DataRef::<i32>::find(path) {
            return Some(DataValue::I32(DataRead::<i32>::get(&r)));
        }
        if let Ok(r) = DataRef::<i16>::find(path) {
            return Some(DataValue::I16(DataRead::<i16>::get(&r)));
        }
        if let Ok(r) = DataRef::<u32>::find(path) {
            return Some(DataValue::U32(DataRead::<u32>::get(&r)));
        }
        if let Ok(r) = DataRef::<u16>::find(path) {
            return Some(DataValue::U16(DataRead::<u16>::get(&r)));
        }
        if let Ok(r) = DataRef::<u8>::find(path) {
            return Some(DataValue::U8(DataRead::<u8>::get(&r)));
        }
        if let Ok(r) = DataRef::<i8>::find(path) {
            return Some(DataValue::I8(DataRead::<i8>::get(&r)));
        }
        if let Ok(r) = DataRef::<bool>::find(path) {
            return Some(DataValue::Bool(DataRead::<bool>::get(&r)));
        }
        None
    }

    pub fn write_ref(&self, path: &str, value: DataValue) -> bool {
        match value {
            DataValue::F32(v) => try_write::<f32>(path, v),
            DataValue::F64(v) => try_write::<f64>(path, v),
            DataValue::I32(v) => try_write::<i32>(path, v),
            DataValue::I16(v) => try_write::<i16>(path, v),
            DataValue::U32(v) => try_write::<u32>(path, v),
            DataValue::U16(v) => try_write::<u16>(path, v),
            DataValue::U8(v) => try_write::<u8>(path, v),
            DataValue::I8(v) => try_write::<i8>(path, v),
            DataValue::Bool(v) => try_write::<bool>(path, v),
        }
    }
}

fn try_write<T: DataType>(path: &str, value: T) -> bool
where
    DataRef<T, ReadOnly>: DataRead<T>,
    DataRef<T, ReadWrite>: DataReadWrite<T>,
{
    if let Ok(r) = DataRef::<T>::find(path) {
        if let Ok(mut w) = r.writeable() {
            DataReadWrite::<T>::set(&mut w, value);
            return true;
        }
    }
    false
}
