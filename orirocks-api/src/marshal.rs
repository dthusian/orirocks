use std::collections::HashMap;
use std::ptr::{null};
use std::slice::from_raw_parts;
use crate::ffi::{Array, Param};

pub fn marshal_params(params: &HashMap<String, String>) -> Vec<Param> {
  params.iter().map(|v| Param {
    key: Array {
      ptr: v.0.as_ptr(),
      len: v.0.bytes().len() as u64
    },
    value: Array {
      ptr: v.1.as_ptr(),
      len: v.1.bytes().len() as u64
    }
  }).collect()
}

pub fn marshal_res(res: Result<(), &'static str>) -> Array<u8> {
  match res {
    Ok(_) => Array { ptr: null(), len: 0 },
    Err(err) => Array { ptr: err.as_ptr(), len: err.bytes().len() as u64 }
  }
}

pub unsafe fn unmarshal_params(params: Array<Param>) -> HashMap<String, String> {
  (0..params.len)
    .map(|i| params.ptr.offset(i as isize).read())
    .map(|v| (
      unmarshal_string(v.key),
      unmarshal_string(v.value)
    ))
    .collect::<HashMap<_, _>>()
}

pub unsafe fn unmarshal_string(s: Array<u8>) -> String {
  String::from_utf8_lossy(from_raw_parts(s.ptr, s.len as usize)).to_string()
}

pub unsafe fn unmarshal_err(err: Array<u8>) -> Result<(), String> {
  if err.ptr != null() {
    Err(unmarshal_string(err))
  } else {
    Ok(())
  }
}