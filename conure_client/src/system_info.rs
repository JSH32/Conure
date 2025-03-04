use conure_common::system_info::SystemInfo;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

/// Collects system information.
pub fn collect_system_info() -> Result<SystemInfo, Box<dyn std::error::Error>> {
    let os_version = System::os_version().unwrap_or("unknown".to_owned());

    // Get current Unix timestamp
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    // Get timezone
    let time_zone = iana_time_zone::get_timezone().unwrap_or("unknown".to_owned());

    Ok(SystemInfo {
        client_id: "huh".to_owned(),
        hostname: whoami::fallible::hostname().unwrap_or("unknown".to_string()),
        os_type: whoami::platform().to_string(),
        os_version,
        os_arch: whoami::arch().to_string(),
        current_time,
        time_zone,
        user_name: whoami::username(),
    })
}
