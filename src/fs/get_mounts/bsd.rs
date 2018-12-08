// Wrapper for the BSD getmntinfo() API which returns a list of mountpoints

use std::io::{Error, Result};
use std::ptr;
use std::slice;
use std::path::PathBuf;
use std::ffi::{CStr, OsStr};
use std::os::unix::ffi::OsStrExt;

use libc::{c_int, statfs};

pub static MNT_NOWAIT: i32 = 2;

extern "C" {
    #[cfg_attr(target_os = "macos", link_name = "getmntinfo$INODE64")]
    fn getmntinfo(mntbufp: *mut *mut statfs, flags: c_int) -> c_int;
}

pub fn get_mount_points() -> Result<Vec<(PathBuf,String)>> {
    let mut raw_mounts_ptr: *mut statfs = ptr::null_mut();

    let rc = unsafe { getmntinfo(&mut raw_mounts_ptr, MNT_NOWAIT) };

    // getmntinfo() has non-obvious error handling behaviour: rc 0 indicates an
    // error (presumably because any Unix system should have at least the root
    // filesystem), requiring us to check errno for the actual error code. The
    // man pages for FreeBSD and Darwin do not acknowledge the possibility of a
    // negative return code so we'll simply panic if that happens.

    if rc == 0 {
        return Err(Error::last_os_error());
    }

    assert!(rc > 0, "getmntinfo() returned undocumented value: {}", rc);

    assert!(
        !raw_mounts_ptr.is_null(),
        "getmntinfo() returned a null pointer to the list of mountpoints!"
    );

    let raw_mounts = unsafe { slice::from_raw_parts(raw_mounts_ptr, rc as usize) };

    let mounts = raw_mounts
        .iter()
        .map(|m| unsafe {(
            let bytes = CStr::from_ptr(&m.f_mntonname[0]).to_bytes();
            PathBuf::from(OsStr::from_bytes(bytes).to_owned()),
            let fstype = CStr::from_ptr(&m.f_fstypename[0]).to_str().unwrap().to_owned()
        }))
        .collect();

    Ok(mounts)
}
