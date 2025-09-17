use crate::{
    backend::{
        c,
        conv::ret_c_int,
    },
    io,
};

#[inline]
pub(crate) unsafe fn prctl(
    option: c::c_int,
    arg2: *mut c::c_void,
    arg3: *mut c::c_void,
    arg4: *mut c::c_void,
    arg5: *mut c::c_void,
) -> io::Result<c::c_int> {
    ret_c_int(c::prctl(option, arg2, arg3, arg4, arg5))
}
