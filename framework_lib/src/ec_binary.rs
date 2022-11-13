const CROS_EC_IMAGE_DATA_COOKIE1: u32 = 0xce778899;
const CROS_EC_IMAGE_DATA_COOKIE2: u32 = 0xceaabbdd;
// Absolute offset of the version struct inside the entire EC binary
const EC_VERSION_OFFSET: usize = 0x1158;

#[cfg(not(feature = "uefi"))]
#[cfg(feature = "std")]
use regex;

#[cfg(feature = "uefi")]
use core::prelude::rust_2021::derive;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct _ImageVersionData {
    cookie1: u32,
    version: [u8; 32],
    size: u32,
    rollback_version: u32,
    cookie2: u32,
}
/// Version Information about an EC FW binary
#[derive(Debug, PartialEq)]
pub struct ImageVersionData {
    /// Full version string, example: hx30_v0.0.1-7a61a89
    pub version: String,
    /// Just the platform/board name, example: hx30
    pub platform: String,
    /// Major part of the version. X of X.Y.Z
    pub major: u32,
    /// Minor part of the version. X of X.Y.Z
    pub minor: u32,
    /// Patch part of the version. X of X.Y.Z
    pub patch: u32,
    /// Commit hash the firmware was built from
    pub commit: String,
    /// TODO: Find out exactly what this is
    pub size: u32,
    /// TODO: Find out exactly what this is
    pub rollback_version: u32,
}

/// Print pretty information about the EC version
pub fn print_ec_version(ver: &ImageVersionData) {
    println!("EC");
    println!("  Version:    {:>20}", ver.version);
    println!("  RollbackVer:{:>20}", ver.rollback_version);
    println!("  Platform:   {:>20}", ver.platform);
    let version = format!("{}.{}.{}", ver.major, ver.minor, ver.patch);
    println!("  Version:    {:>20}", version);
    println!("  Commit:     {:>20}", ver.commit);
    println!("  Size:       {:>20} B", ver.size);
    println!("  Size:       {:>20} KB", ver.size / 1024);
}

#[cfg(feature = "uefi")]
fn parse_ec_version(data: &_ImageVersionData) -> Option<ImageVersionData> {
    let version = std::str::from_utf8(&data.version)
        .ok()?
        .trim_end_matches(char::from(0));

    // TODO: regex crate does not support no_std

    Some(ImageVersionData {
        version: version.to_string(),
        size: data.size,
        rollback_version: data.rollback_version,
        platform: "".to_string(),
        major: 0,
        minor: 0,
        patch: 0,
        commit: "".to_string(),
    })
}

#[cfg(not(feature = "uefi"))]
fn parse_ec_version(data: &_ImageVersionData) -> Option<ImageVersionData> {
    let version = std::str::from_utf8(&data.version)
        .ok()?
        .trim_end_matches(char::from(0));
    // Example: hx30_v0.0.1-7a61a89
    let re = regex::Regex::new(r"([a-z0-9]+)_v([0-9])\.([0-9])\.([0-9])-([0-9a-f]+)").unwrap();
    let caps = re.captures(version).unwrap();
    let platform = caps.get(1)?.as_str().to_string();
    let major = caps.get(2)?.as_str().parse::<u32>().ok()?;
    let minor = caps.get(3)?.as_str().parse::<u32>().ok()?;
    let patch = caps.get(4)?.as_str().parse::<u32>().ok()?;
    let commit = caps.get(5)?.as_str().to_string();

    Some(ImageVersionData {
        version: version.to_string(),
        size: data.size,
        rollback_version: data.rollback_version,
        platform,
        major,
        minor,
        patch,
        commit,
    })
}

/// Parse version information from EC FW image buffer
pub fn read_ec_version(data: &[u8]) -> Option<ImageVersionData> {
    let v: _ImageVersionData =
        unsafe { std::ptr::read(data[EC_VERSION_OFFSET..].as_ptr() as *const _) };
    if v.cookie1 != CROS_EC_IMAGE_DATA_COOKIE1 {
        println!("Failed to find Cookie 1");
        return None;
    }
    if v.cookie2 != CROS_EC_IMAGE_DATA_COOKIE2 {
        println!("Failed to find Cookie 2");
        return None;
    }

    parse_ec_version(&v)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Perhaps put the binary hex data here and test it all
    #[test]
    fn can_parse() {
        let ver_chars: &[u8] = b"hx30_v0.0.1-7a61a89\0\0\0\0\0\0\0\0\0\0\0\0\0";
        let data = _ImageVersionData {
            cookie1: CROS_EC_IMAGE_DATA_COOKIE1,
            version: ver_chars.try_into().unwrap(),
            size: 2868,
            rollback_version: 0,
            cookie2: CROS_EC_IMAGE_DATA_COOKIE1,
        };
        debug_assert_eq!(
            parse_ec_version(&data),
            Some(ImageVersionData {
                version: "hx30_v0.0.1-7a61a89".to_string(),
                size: 2868,
                rollback_version: 0,
                platform: "hx30".to_string(),
                major: 0,
                minor: 0,
                patch: 1,
                commit: "7a61a89".to_string(),
            })
        );
    }
}
