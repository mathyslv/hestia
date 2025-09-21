pub mod process;

use crate::Result;

pub fn install_callbacks() -> Result<()> {
    process::install_create_process_callback()?;
    Ok(())
}

pub fn uninstall_callbacks() -> Result<()> {
    process::uninstall_create_process_callback()?;
    Ok(())
}
