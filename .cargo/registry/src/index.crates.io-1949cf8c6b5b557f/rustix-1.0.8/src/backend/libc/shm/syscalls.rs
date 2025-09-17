use crate::ffi::CStr;

use crate::{
    backend::{
        c,
        conv::{
            c_str,
            ret,
            ret_owned_fd,
        },
    },
    fd::OwnedFd,
    fs::Mode,
    io,
    shm,
};

pub(crate) fn shm_open(name: &CStr, oflags: shm::OFlags, mode: Mode) -> io::Result<OwnedFd> {
    // On this platforms, `mode_t` is `u16` and can't be passed directly to a
    // variadic function.
    #[cfg(apple)]
    let mode: c::c_uint = mode.bits().into();

    // Otherwise, cast to `mode_t` as that's what `open` is documented to take.
    #[cfg(not(apple))]
    let mode: c::mode_t = mode.bits() as _;

    unsafe { ret_owned_fd(c::shm_open(c_str(name), bitflags_bits!(oflags), mode)) }
}

pub(crate) fn shm_unlink(name: &CStr) -> io::Result<()> {
    unsafe { ret(c::shm_unlink(c_str(name))) }
}
