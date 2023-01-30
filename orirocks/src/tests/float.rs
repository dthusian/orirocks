use std::cmp::Ordering;
use crate::float::CmpFloat;

#[test]
fn compare_nan_1() {
  assert_ne!(CmpFloat::new(4.5), CmpFloat::new(f64::NAN));
  assert_eq!(CmpFloat::new(4.5).cmp(&CmpFloat::new(f64::NAN)), Ordering::Greater);
}

#[test]
fn compare_nan_2() {
  assert_eq!(CmpFloat::new(f64::NAN), CmpFloat::new(f64::NAN));
  assert_eq!(CmpFloat::new(f64::NAN).cmp(&CmpFloat::new(f64::NAN)), Ordering::Equal);
}