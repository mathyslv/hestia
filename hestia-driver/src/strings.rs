#[macro_export]
macro_rules! make_unicode_string {
    ($name:ident, $s:literal) => {
        let pcwstr = w!($s);
        let mut $name: UNICODE_STRING = unsafe { core::mem::zeroed() };
        unsafe { RtlInitUnicodeString(&mut $name, pcwstr.0) };
    };
}
