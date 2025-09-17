macro_rules! force_eval {
    ($e:expr) => {
        unsafe { ::core::ptr::read_volatile(&$e) }
    };
}

#[cfg(not(debug_assertions))]
macro_rules! i {
    ($array:expr, $index:expr) => {
        unsafe { *$array.get_unchecked($index) }
    };
    ($array:expr, $index:expr, = , $rhs:expr) => {
        unsafe {
            *$array.get_unchecked_mut($index) = $rhs;
        }
    };
    ($array:expr, $index:expr, += , $rhs:expr) => {
        unsafe {
            *$array.get_unchecked_mut($index) += $rhs;
        }
    };
    ($array:expr, $index:expr, -= , $rhs:expr) => {
        unsafe {
            *$array.get_unchecked_mut($index) -= $rhs;
        }
    };
    ($array:expr, $index:expr, &= , $rhs:expr) => {
        unsafe {
            *$array.get_unchecked_mut($index) &= $rhs;
        }
    };
    ($array:expr, $index:expr, == , $rhs:expr) => {
        unsafe { *$array.get_unchecked_mut($index) == $rhs }
    };
}

#[cfg(debug_assertions)]
macro_rules! i {
    ($array:expr, $index:expr) => {
        *$array.get($index).unwrap()
    };
    ($array:expr, $index:expr, = , $rhs:expr) => {
        *$array.get_mut($index).unwrap() = $rhs;
    };
    ($array:expr, $index:expr, -= , $rhs:expr) => {
        *$array.get_mut($index).unwrap() -= $rhs;
    };
    ($array:expr, $index:expr, += , $rhs:expr) => {
        *$array.get_mut($index).unwrap() += $rhs;
    };
    ($array:expr, $index:expr, &= , $rhs:expr) => {
        *$array.get_mut($index).unwrap() &= $rhs;
    };
    ($array:expr, $index:expr, == , $rhs:expr) => {
        *$array.get_mut($index).unwrap() == $rhs
    };
}

// Temporary macro to avoid panic codegen for division (in debug mode too). At
// the time of this writing this is only used in a few places, and once
// rust-lang/rust#72751 is fixed then this macro will no longer be necessary and
// the native `/` operator can be used and panics won't be codegen'd.
#[cfg(any(debug_assertions, not(intrinsics_enabled)))]
macro_rules! div {
    ($a:expr, $b:expr) => {
        $a / $b
    };
}

#[cfg(all(not(debug_assertions), intrinsics_enabled))]
macro_rules! div {
    ($a:expr, $b:expr) => {
        unsafe { core::intrinsics::unchecked_div($a, $b) }
    };
}

// `support` may be public for testing
#[macro_use]
#[cfg(feature = "unstable-public-internals")]
pub mod support;

#[macro_use]
#[cfg(not(feature = "unstable-public-internals"))]
pub(crate) mod support;

cfg_if! {
    if #[cfg(feature = "unstable-public-internals")] {
        pub mod generic;
    } else {
        mod generic;
    }
}

// Private modules
mod arch;
mod expo2;
mod k_cos;
mod k_cosf;
mod k_expo2;
mod k_expo2f;
mod k_sin;
mod k_sinf;
mod k_tan;
mod k_tanf;
mod rem_pio2;
mod rem_pio2_large;
mod rem_pio2f;

// Private re-imports
#[allow(unused_imports)]
use self::support::{
    CastFrom,
    CastInto,
    DFloat,
    DInt,
    Float,
    HFloat,
    HInt,
    Int,
    IntTy,
    MinInt,
};
use self::{
    expo2::expo2,
    k_cos::k_cos,
    k_cosf::k_cosf,
    k_expo2::k_expo2,
    k_expo2f::k_expo2f,
    k_sin::k_sin,
    k_sinf::k_sinf,
    k_tan::k_tan,
    k_tanf::k_tanf,
    rem_pio2::rem_pio2,
    rem_pio2_large::rem_pio2_large,
    rem_pio2f::rem_pio2f,
};

// Public modules
mod acos;
mod acosf;
mod acosh;
mod acoshf;
mod asin;
mod asinf;
mod asinh;
mod asinhf;
mod atan;
mod atan2;
mod atan2f;
mod atanf;
mod atanh;
mod atanhf;
mod cbrt;
mod cbrtf;
mod ceil;
mod copysign;
mod cos;
mod cosf;
mod cosh;
mod coshf;
mod erf;
mod erff;
mod exp;
mod exp10;
mod exp10f;
mod exp2;
mod exp2f;
mod expf;
mod expm1;
mod expm1f;
mod fabs;
mod fdim;
mod floor;
mod fma;
mod fmin_fmax;
mod fminimum_fmaximum;
mod fminimum_fmaximum_num;
mod fmod;
mod frexp;
mod frexpf;
mod hypot;
mod hypotf;
mod ilogb;
mod ilogbf;
mod j0;
mod j0f;
mod j1;
mod j1f;
mod jn;
mod jnf;
mod ldexp;
mod lgamma;
mod lgamma_r;
mod lgammaf;
mod lgammaf_r;
mod log;
mod log10;
mod log10f;
mod log1p;
mod log1pf;
mod log2;
mod log2f;
mod logf;
mod modf;
mod modff;
mod nextafter;
mod nextafterf;
mod pow;
mod powf;
mod remainder;
mod remainderf;
mod remquo;
mod remquof;
mod rint;
mod round;
mod roundeven;
mod scalbn;
mod sin;
mod sincos;
mod sincosf;
mod sinf;
mod sinh;
mod sinhf;
mod sqrt;
mod tan;
mod tanf;
mod tanh;
mod tanhf;
mod tgamma;
mod tgammaf;
mod trunc;

// Use separated imports instead of {}-grouped imports for easier merging.
pub use self::{
    acos::acos,
    acosf::acosf,
    acosh::acosh,
    acoshf::acoshf,
    asin::asin,
    asinf::asinf,
    asinh::asinh,
    asinhf::asinhf,
    atan::atan,
    atan2::atan2,
    atan2f::atan2f,
    atanf::atanf,
    atanh::atanh,
    atanhf::atanhf,
    cbrt::cbrt,
    cbrtf::cbrtf,
    ceil::{
        ceil,
        ceilf,
    },
    copysign::{
        copysign,
        copysignf,
    },
    cos::cos,
    cosf::cosf,
    cosh::cosh,
    coshf::coshf,
    erf::{
        erf,
        erfc,
    },
    erff::{
        erfcf,
        erff,
    },
    exp::exp,
    exp10::exp10,
    exp10f::exp10f,
    exp2::exp2,
    exp2f::exp2f,
    expf::expf,
    expm1::expm1,
    expm1f::expm1f,
    fabs::{
        fabs,
        fabsf,
    },
    fdim::{
        fdim,
        fdimf,
    },
    floor::{
        floor,
        floorf,
    },
    fma::{
        fma,
        fmaf,
    },
    fmin_fmax::{
        fmax,
        fmaxf,
        fmin,
        fminf,
    },
    fminimum_fmaximum::{
        fmaximum,
        fmaximumf,
        fminimum,
        fminimumf,
    },
    fminimum_fmaximum_num::{
        fmaximum_num,
        fmaximum_numf,
        fminimum_num,
        fminimum_numf,
    },
    fmod::{
        fmod,
        fmodf,
    },
    frexp::frexp,
    frexpf::frexpf,
    hypot::hypot,
    hypotf::hypotf,
    ilogb::ilogb,
    ilogbf::ilogbf,
    j0::{
        j0,
        y0,
    },
    j0f::{
        j0f,
        y0f,
    },
    j1::{
        j1,
        y1,
    },
    j1f::{
        j1f,
        y1f,
    },
    jn::{
        jn,
        yn,
    },
    jnf::{
        jnf,
        ynf,
    },
    ldexp::{
        ldexp,
        ldexpf,
    },
    lgamma::lgamma,
    lgamma_r::lgamma_r,
    lgammaf::lgammaf,
    lgammaf_r::lgammaf_r,
    log::log,
    log10::log10,
    log10f::log10f,
    log1p::log1p,
    log1pf::log1pf,
    log2::log2,
    log2f::log2f,
    logf::logf,
    modf::modf,
    modff::modff,
    nextafter::nextafter,
    nextafterf::nextafterf,
    pow::pow,
    powf::powf,
    remainder::remainder,
    remainderf::remainderf,
    remquo::remquo,
    remquof::remquof,
    rint::{
        rint,
        rintf,
    },
    round::{
        round,
        roundf,
    },
    roundeven::{
        roundeven,
        roundevenf,
    },
    scalbn::{
        scalbn,
        scalbnf,
    },
    sin::sin,
    sincos::sincos,
    sincosf::sincosf,
    sinf::sinf,
    sinh::sinh,
    sinhf::sinhf,
    sqrt::{
        sqrt,
        sqrtf,
    },
    tan::tan,
    tanf::tanf,
    tanh::tanh,
    tanhf::tanhf,
    tgamma::tgamma,
    tgammaf::tgammaf,
    trunc::{
        trunc,
        truncf,
    },
};

cfg_if! {
    if #[cfg(f16_enabled)] {
        // verify-sorted-start
        pub use self::ceil::ceilf16;
        pub use self::copysign::copysignf16;
        pub use self::fabs::fabsf16;
        pub use self::fdim::fdimf16;
        pub use self::floor::floorf16;
        pub use self::fmin_fmax::{fmaxf16, fminf16};
        pub use self::fminimum_fmaximum::{fmaximumf16, fminimumf16};
        pub use self::fminimum_fmaximum_num::{fmaximum_numf16, fminimum_numf16};
        pub use self::fmod::fmodf16;
        pub use self::ldexp::ldexpf16;
        pub use self::rint::rintf16;
        pub use self::round::roundf16;
        pub use self::roundeven::roundevenf16;
        pub use self::scalbn::scalbnf16;
        pub use self::sqrt::sqrtf16;
        pub use self::trunc::truncf16;
        // verify-sorted-end

        #[allow(unused_imports)]
        pub(crate) use self::fma::fmaf16;
    }
}

cfg_if! {
    if #[cfg(f128_enabled)] {
        // verify-sorted-start
        pub use self::ceil::ceilf128;
        pub use self::copysign::copysignf128;
        pub use self::fabs::fabsf128;
        pub use self::fdim::fdimf128;
        pub use self::floor::floorf128;
        pub use self::fma::fmaf128;
        pub use self::fmin_fmax::{fmaxf128, fminf128};
        pub use self::fminimum_fmaximum::{fmaximumf128, fminimumf128};
        pub use self::fminimum_fmaximum_num::{fmaximum_numf128, fminimum_numf128};
        pub use self::fmod::fmodf128;
        pub use self::ldexp::ldexpf128;
        pub use self::rint::rintf128;
        pub use self::round::roundf128;
        pub use self::roundeven::roundevenf128;
        pub use self::scalbn::scalbnf128;
        pub use self::sqrt::sqrtf128;
        pub use self::trunc::truncf128;
        // verify-sorted-end
    }
}

#[inline]
fn get_high_word(x: f64) -> u32 {
    (x.to_bits() >> 32) as u32
}

#[inline]
fn get_low_word(x: f64) -> u32 {
    x.to_bits() as u32
}

#[inline]
fn with_set_high_word(f: f64, hi: u32) -> f64 {
    let mut tmp = f.to_bits();
    tmp &= 0x00000000_ffffffff;
    tmp |= (hi as u64) << 32;
    f64::from_bits(tmp)
}

#[inline]
fn with_set_low_word(f: f64, lo: u32) -> f64 {
    let mut tmp = f.to_bits();
    tmp &= 0xffffffff_00000000;
    tmp |= lo as u64;
    f64::from_bits(tmp)
}

#[inline]
fn combine_words(hi: u32, lo: u32) -> f64 {
    f64::from_bits(((hi as u64) << 32) | lo as u64)
}
