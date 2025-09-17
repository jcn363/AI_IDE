use std::{
    env,
    fs::OpenOptions,
    io::{
        BufRead,
        BufReader,
    },
};

pub(crate) fn get_timezone_inner() -> Result<String, crate::GetTimezoneError> {
    env::var("TZ").map_err(|_| crate::GetTimezoneError::OsError)
}
