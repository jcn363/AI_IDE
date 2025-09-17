//! Extension traits.

mod digit_count;
#[cfg(feature = "std")]
mod instant;
mod numerical_duration;
mod numerical_std_duration;
#[cfg(feature = "std")]
mod systemtime;

pub(crate) use self::digit_count::DigitCount;
#[cfg(feature = "std")]
pub use self::instant::InstantExt;
#[cfg(feature = "std")]
pub use self::systemtime::SystemTimeExt;
pub use self::{
    numerical_duration::NumericalDuration,
    numerical_std_duration::NumericalStdDuration,
};
