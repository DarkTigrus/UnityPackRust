/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

#[derive(Debug)]
pub enum RuntimePlatform {
    OSXEditor,
    OSXPlayer,
    WindowsPlayer,
    OSXWebPlayer,
    OSXDashboardPlayer,
    WindowsWebPlayer,
    WindowsEditor,
    IPhonePlayer,
    PS3,
    XBOX360,
    Android,
    NaCl,
    LinuxPlayer,
    FlashPlayer,
    WebGLPlayer,
    MetroPlayerX86,
    MetroPlayerX64,
    MetroPlayerARM,
    WP8Player,
    BB10Player,
    TizenPlayer,
    PSP2,
    PS4,
    PSM,
    XboxOne,
    SamsungTVPlayer,
}

pub fn get_runtime_platform(value: u32) -> RuntimePlatform {
    match value {
        0 => RuntimePlatform::OSXEditor,
        1 => RuntimePlatform::OSXPlayer,
        2 => RuntimePlatform::WindowsPlayer,
        3 => RuntimePlatform::OSXWebPlayer,
        4 => RuntimePlatform::OSXDashboardPlayer,
        5 => RuntimePlatform::WindowsWebPlayer,
        7 => RuntimePlatform::WindowsEditor,
        8 => RuntimePlatform::IPhonePlayer,
        9 => RuntimePlatform::PS3,
        10 => RuntimePlatform::XBOX360,
        11 => RuntimePlatform::Android,
        12 => RuntimePlatform::NaCl,
        13 => RuntimePlatform::LinuxPlayer,
        15 => RuntimePlatform::FlashPlayer,
        17 => RuntimePlatform::WebGLPlayer,
        18 => RuntimePlatform::MetroPlayerX86,
        19 => RuntimePlatform::MetroPlayerX64,
        20 => RuntimePlatform::MetroPlayerARM,
        21 => RuntimePlatform::WP8Player,
        22 => RuntimePlatform::BB10Player,
        23 => RuntimePlatform::TizenPlayer,
        24 => RuntimePlatform::PSP2,
        25 => RuntimePlatform::PS4,
        26 => RuntimePlatform::PSM,
        27 => RuntimePlatform::XboxOne,
        28 => RuntimePlatform::SamsungTVPlayer,
        _ => RuntimePlatform::OSXEditor,
    }
}
