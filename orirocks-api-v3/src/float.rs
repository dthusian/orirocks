use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

/// A comparable float value
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(transparent)]
pub struct CmpFloat {
  pub inner: f64
}

impl CmpFloat {
  pub fn new(f: f64) -> Self {
    CmpFloat {
      inner: f
    }
  }
}

impl Into<f64> for CmpFloat {
  fn into(self) -> f64 {
    self.inner
  }
}

impl PartialEq<Self> for CmpFloat {
  fn eq(&self, other: &Self) -> bool {
    if self.inner.is_nan() && other.inner.is_nan() {
      true
    } else if self.inner.is_nan() {
      false
    } else if other.inner.is_nan() {
      false
    } else {
      self.inner.eq(&other.inner)
    }
  }
}

impl Eq for CmpFloat { }

impl PartialOrd for CmpFloat {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(if self.inner.is_nan() && other.inner.is_nan() {
      Ordering::Equal
    } else if self.inner.is_nan() {
      Ordering::Less
    } else if other.inner.is_nan() {
      Ordering::Greater
    } else {
      self.inner.partial_cmp(&other.inner).unwrap()
    })
  }
}

impl Ord for CmpFloat {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap()
  }
}