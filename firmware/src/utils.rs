use anyhow::{bail, Result};
use esp_idf_sys::esp_efuse_mac_get_default;

pub fn mac_addr() -> Result<([u8; 6])> {
    let mut mac = [0_u8; 6];
    let ret = unsafe { esp_efuse_mac_get_default(mac.as_mut_ptr()) };
    if ret != 0 {
        bail!("Cannot get MAC ADDR");
    }

    Ok(mac)
}

pub fn device_id() -> Result<u32> {
    let mac = mac_addr()?;
    let v1 = mac[0] as u32 | (mac[1] as u32) << 8 | (mac[2] as u32) << 16 | (mac[3] as u32) << 24;
    let v2 = mac[4] as u32 | (mac[5] as u32) << 8;
    Ok(v1 ^ v2)
}
