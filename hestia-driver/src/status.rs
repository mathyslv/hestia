use core::ops::Deref;

use wdk::nt_success;
use wdk_sys::NTSTATUS;

pub struct NtStatus(NTSTATUS);

impl Deref for NtStatus {
    type Target = NTSTATUS;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NtStatus> for NTSTATUS {
    fn from(value: NtStatus) -> Self {
        value.0
    }
}

pub type Result<T> = ::core::result::Result<T, NTSTATUS>;

impl From<NtStatus> for Result<()> {
    fn from(value: NtStatus) -> Self {
        if nt_success(*value) {
            Ok(())
        } else {
            Err(value.0)
        }
    }
}
