use shared::{
    minwindef::BOOL,
    ntdef::{
        PHANDLE,
        ULONG,
    },
};
//1286
extern "system" {
    pub fn WTSQueryUserToken(SessionId: ULONG, phToken: PHANDLE) -> BOOL;
}
