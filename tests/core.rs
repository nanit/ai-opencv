use std::mem;

use opencv::core::{Moments, Point2f, RotatedRect, Scalar, Size2f, Vec3b, Vector};
use opencv::prelude::*;
use opencv::{core, Result};

#[test]
fn cv_return_type() {
	assert_eq!(mem::size_of::<opencv::sys::Result<i32>>(), 24);
	assert_eq!(mem::size_of::<opencv::sys::Result<u8>>(), 24);
	assert_eq!(mem::size_of::<opencv::sys::Result<()>>(), 16);
}

#[test]
fn moments() -> Result<()> {
	let moments = Moments::default()?;
	assert_eq!(0., moments.m00);
	assert_eq!(0., moments.m12);
	assert_eq!(0., moments.mu30);
	Ok(())
}

#[test]
fn cpu_features_line() -> Result<()> {
	let cpu_feats = core::get_cpu_features_line()?;
	assert!(cpu_feats.is_ascii());
	Ok(())
}

#[test]
fn rotated_rect() -> Result<()> {
	let rect = RotatedRect::new(Point2f::new(100., 100.), Size2f::new(100., 100.), 90.)?;
	let mut pts = [Point2f::default(); 4];
	rect.points(&mut pts)?;
	assert_eq!(Point2f::new(50., 50.), pts[0]);
	assert_eq!(Point2f::new(150., 50.), pts[1]);
	assert_eq!(Point2f::new(150., 150.), pts[2]);
	assert_eq!(Point2f::new(50., 150.), pts[3]);

	Ok(())
}

#[test]
fn in_range() -> Result<()> {
	let mut cs = Vector::<Mat>::new();
	cs.push(Mat::from_slice_2d(&[&[1., 2., 3.], &[4., 5., 6.], &[7., 8., 9.]])?);
	cs.push(Mat::from_slice_2d(&[&[11., 12., 13.], &[14., 15., 16.], &[17., 18., 19.]])?);
	let mut m = Mat::default();
	core::merge(&cs, &mut m)?;
	let mut out = Mat::default();
	let lowerb = Scalar::from((2, 10));
	let upperb = Scalar::from((6, 15));
	core::in_range(&m, &lowerb, &upperb, &mut out)?;

	assert_eq!(&[0, 255, 255, 255, 255, 0, 0, 0, 0], &out.data_typed::<u8>()?);
	Ok(())
}

#[test]
#[cfg(not(ocvrs_opencv_branch_34))]
fn file_storage() -> Result<()> {
	use opencv::core::{FileStorage, FileStorage_Mode};

	let mut st = FileStorage::new_def(
		".yml",
		i32::from(FileStorage_Mode::WRITE) | i32::from(FileStorage_Mode::MEMORY),
	)?;
	st.write_i32("test_int", 98)?;
	core::write_f64(&mut st, "test_double", 123.45)?;
	st.write_str("test_str", "test string")?;
	let serialized = st.release_and_get_string()?;

	let st = FileStorage::new_def(&serialized, FileStorage_Mode::MEMORY.into())?;
	let int_node = st.get("test_int")?;
	assert!(int_node.is_int()?);
	assert_eq!(98, int_node.to_i32()?);
	let double_node = st.get_node("test_double")?;
	assert!(double_node.is_real()?);
	assert_eq!(123.45, double_node.to_f64()?);
	let str_node = st.get("test_str")?;
	assert!(str_node.is_string()?);
	assert_eq!("test string", str_node.to_string()?);

	let mut str_out = String::new();
	core::read_str(&str_node, &mut str_out, "default string")?;
	assert_eq!("test string", str_out);
	core::read_str(&st.get("non_existent")?, &mut str_out, "default string")?;
	assert_eq!("default string", str_out);

	Ok(())
}

/// Make sure that arguments to min_max_loc are nullable
#[test]
fn min_max_loc() -> Result<()> {
	let mut m = Mat::new_rows_cols_with_default(10, 10, Vec3b::opencv_type(), Scalar::all(5.))?;
	let (mut min_val, mut max_val) = (90., 90.);
	*m.at_2d_mut(5, 5)? = Vec3b::from([10, 20, 30]);
	core::min_max_loc(&m, Some(&mut min_val), Some(&mut max_val), None, None, &core::no_array())?;
	assert_eq!(5., min_val);
	assert_eq!(30., max_val);
	Ok(())
}
