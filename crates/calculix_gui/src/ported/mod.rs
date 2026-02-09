//! First migrated routines from `cgx_2.23/src`.

mod scalar;
mod string;
mod vector;

pub use scalar::{check_if_number, p_angle};
pub use string::{compare_prefix, compare_strings, strfind};
pub use vector::{v_add, v_angle, v_norm, v_prod, v_result, v_sprod};
