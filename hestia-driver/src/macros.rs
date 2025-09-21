/// Rust port of the DEFINE_GUID C macro.
///
/// It is compatible with output of Visual Studio's 'Create GUID' tool (only need to add `!` for macro invocation).
///
/// ## Examples
///
/// ```rust,norun
/// // {509969B5-B0E0-452C-A372-A9A754D0423C}
/// DEFINE_GUID!(
///     HESTIA_GUID,
///     0x509969b5, 0xb0e0, 0x452c, 0xa3, 0x72, 0xa9, 0xa7, 0x54, 0xd0, 0x42, 0x3c
/// );
/// ```
///
/// Cf. https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/defining-and-exporting-new-guids
#[macro_export]
macro_rules! DEFINE_GUID {
    (
        $name:ident,
        $d1:literal,
        $d2:literal,
        $d3:literal,
        $d4:literal, $d5:literal, $d6:literal, $d7:literal, $d8:literal, $d9:literal, $d10:literal, $d11:literal) => {
        const $name: GUID = GUID {
            Data1: $d1,
            Data2: $d2,
            Data3: $d3,
            Data4: [$d4, $d5, $d6, $d7, $d8, $d9, $d10, $d11],
        };
    };
}
