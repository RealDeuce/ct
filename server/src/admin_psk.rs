//! Administrator PSK file creation and loading.

use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub const ADMIN_PSK_BYTES: usize = 32;

pub fn load_or_create(path: &Path) -> io::Result<Vec<u8>> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "PSK path has no parent"))?;
    fs::create_dir_all(parent)?;
    secure_directory(parent)?;

    let mut created = false;
    let mut file = match secure_create(path) {
        Ok(file) => {
            created = true;
            file
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            secure_existing_file(path)?;
            OpenOptions::new().read(true).open(path)?
        }
        Err(error) => return Err(error),
    };

    if created {
        let mut key = vec![0; ADMIN_PSK_BYTES];
        getrandom::fill(&mut key).map_err(io::Error::other)?;
        file.write_all(&key)?;
        file.sync_all()?;
        Ok(key)
    } else {
        let mut key = Vec::new();
        file.read_to_end(&mut key)?;
        if key.len() != ADMIN_PSK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("administrator PSK must contain exactly {ADMIN_PSK_BYTES} bytes"),
            ));
        }
        Ok(key)
    }
}

#[cfg(unix)]
fn secure_create(path: &Path) -> io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(unix)]
fn secure_existing_file(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(unix)]
fn secure_directory(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(windows)]
fn secure_create(path: &Path) -> io::Result<std::fs::File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)?;
    apply_windows_dacl(path, false)?;
    Ok(file)
}

#[cfg(windows)]
fn secure_existing_file(path: &Path) -> io::Result<()> {
    apply_windows_dacl(path, false)
}

#[cfg(windows)]
fn secure_directory(path: &Path) -> io::Result<()> {
    apply_windows_dacl(path, true)
}

#[cfg(windows)]
fn apply_windows_dacl(path: &Path, directory: bool) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::{
        DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, SetFileSecurityW,
    };

    let mut path_wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    path_wide.push(0);
    let sddl = if directory {
        "D:P(A;OICI;FA;;;OW)(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)"
    } else {
        "D:P(A;;FA;;;OW)(A;;FA;;;SY)(A;;FA;;;BA)"
    };
    let mut sddl_wide: Vec<u16> = sddl.encode_utf16().collect();
    sddl_wide.push(0);
    let mut descriptor = ptr::null_mut();
    // SAFETY: both strings are terminated and descriptor is released with LocalFree.
    let converted = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl_wide.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            ptr::null_mut(),
        )
    };
    if converted == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: descriptor contains a valid self-relative security descriptor.
    let applied = unsafe {
        SetFileSecurityW(
            path_wide.as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            descriptor,
        )
    };
    // SAFETY: descriptor was allocated by the conversion function.
    unsafe {
        LocalFree(descriptor.cast());
    }
    if applied == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
compile_error!("administrator PSK permissions are not implemented for this platform");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_once_and_reloads_the_same_key() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("admin.psk");
        let first = load_or_create(&path).unwrap();
        let second = load_or_create(&path).unwrap();
        assert_eq!(first.len(), ADMIN_PSK_BYTES);
        assert_eq!(first, second);
    }

    #[cfg(unix)]
    #[test]
    fn creates_private_unix_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("server-data");
        let path = directory.join("admin.psk");
        load_or_create(&path).unwrap();
        assert_eq!(
            fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
