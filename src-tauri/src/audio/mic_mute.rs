//! System-wide microphone mute, like the Mic Drop menu bar app.
//!
//! Muting sets the CoreAudio input mute and turns the input volume to 0 on
//! every input device. Both are needed: on macOS 27 the mute property accepts
//! writes but keeps reading 0, while the volume change takes effect (checked
//! on 2026-11-01 with a USB microphone and the built-in microphone). Unmuting
//! restores the saved volume, or UNKNOWN_RESTORE_VOLUME when Talkey did not
//! mute that device itself (for example after a restart). Devices with
//! neither control (some virtual devices) cannot be muted. Other platforms do
//! not support this yet.

/// Whether this platform can mute the microphone system-wide.
pub const SUPPORTED: bool = cfg!(target_os = "macos");

#[cfg(target_os = "macos")]
mod macos {
    use std::collections::HashMap;
    use std::ffi::c_void;
    use std::sync::Mutex;

    type AudioObjectId = u32;
    type OsStatus = i32;

    #[repr(C)]
    struct PropertyAddress {
        selector: u32,
        scope: u32,
        element: u32,
    }

    #[link(name = "CoreAudio", kind = "framework")]
    unsafe extern "C" {
        fn AudioObjectHasProperty(id: AudioObjectId, address: *const PropertyAddress) -> u8;
        fn AudioObjectIsPropertySettable(
            id: AudioObjectId,
            address: *const PropertyAddress,
            settable: *mut u8,
        ) -> OsStatus;
        fn AudioObjectGetPropertyDataSize(
            id: AudioObjectId,
            address: *const PropertyAddress,
            qualifier_size: u32,
            qualifier: *const c_void,
            size: *mut u32,
        ) -> OsStatus;
        fn AudioObjectGetPropertyData(
            id: AudioObjectId,
            address: *const PropertyAddress,
            qualifier_size: u32,
            qualifier: *const c_void,
            size: *mut u32,
            data: *mut c_void,
        ) -> OsStatus;
        fn AudioObjectSetPropertyData(
            id: AudioObjectId,
            address: *const PropertyAddress,
            qualifier_size: u32,
            qualifier: *const c_void,
            size: u32,
            data: *const c_void,
        ) -> OsStatus;
    }

    const fn code(name: &[u8; 4]) -> u32 {
        u32::from_be_bytes(*name)
    }

    const SYSTEM_OBJECT: AudioObjectId = 1;
    const DEVICES: u32 = code(b"dev#");
    const DEFAULT_INPUT_DEVICE: u32 = code(b"dIn ");
    const STREAMS: u32 = code(b"stm#");
    const MUTE: u32 = code(b"mute");
    const VOLUME: u32 = code(b"volm");
    const SCOPE_GLOBAL: u32 = code(b"glob");
    const SCOPE_INPUT: u32 = code(b"inpt");
    const ELEMENT_MAIN: u32 = 0;
    /// Channel elements tried when a device has no main (master) control.
    const MAX_CHANNEL: u32 = 8;
    /// Input volume treated as "off".
    const SILENT_VOLUME: f32 = 0.001;
    /// Volume used on unmute when the volume before muting is unknown.
    const UNKNOWN_RESTORE_VOLUME: f32 = 0.75;

    /// Volumes saved by `set_muted(true)` for devices without a mute control.
    static SAVED_VOLUMES: Mutex<Option<HashMap<(AudioObjectId, u32), f32>>> = Mutex::new(None);

    fn address(selector: u32, scope: u32, element: u32) -> PropertyAddress {
        PropertyAddress {
            selector,
            scope,
            element,
        }
    }

    fn has(id: AudioObjectId, address: &PropertyAddress) -> bool {
        unsafe { AudioObjectHasProperty(id, address) != 0 }
    }

    fn settable(id: AudioObjectId, address: &PropertyAddress) -> bool {
        let mut value = 0u8;
        has(id, address)
            && unsafe { AudioObjectIsPropertySettable(id, address, &mut value) } == 0
            && value != 0
    }

    fn get<T: Copy + Default>(id: AudioObjectId, address: &PropertyAddress) -> Option<T> {
        if !has(id, address) {
            return None;
        }
        let mut value = T::default();
        let mut size = std::mem::size_of::<T>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                id,
                address,
                0,
                std::ptr::null(),
                &mut size,
                (&mut value as *mut T).cast(),
            )
        };
        (status == 0).then_some(value)
    }

    fn set<T: Copy>(id: AudioObjectId, address: &PropertyAddress, value: T) -> bool {
        let status = unsafe {
            AudioObjectSetPropertyData(
                id,
                address,
                0,
                std::ptr::null(),
                std::mem::size_of::<T>() as u32,
                (&value as *const T).cast(),
            )
        };
        status == 0
    }

    fn input_devices() -> Vec<AudioObjectId> {
        let devices_address = address(DEVICES, SCOPE_GLOBAL, ELEMENT_MAIN);
        let mut size = 0u32;
        if unsafe {
            AudioObjectGetPropertyDataSize(
                SYSTEM_OBJECT,
                &devices_address,
                0,
                std::ptr::null(),
                &mut size,
            )
        } != 0
        {
            return Vec::new();
        }
        let mut ids =
            vec![0 as AudioObjectId; size as usize / std::mem::size_of::<AudioObjectId>()];
        if unsafe {
            AudioObjectGetPropertyData(
                SYSTEM_OBJECT,
                &devices_address,
                0,
                std::ptr::null(),
                &mut size,
                ids.as_mut_ptr().cast(),
            )
        } != 0
        {
            return Vec::new();
        }
        ids.into_iter()
            .filter(|&id| {
                let streams = address(STREAMS, SCOPE_INPUT, ELEMENT_MAIN);
                let mut size = 0u32;
                let status = unsafe {
                    AudioObjectGetPropertyDataSize(id, &streams, 0, std::ptr::null(), &mut size)
                };
                status == 0 && size > 0
            })
            .collect()
    }

    /// Elements that carry `selector` on the input side: the main element if
    /// it is settable, otherwise every settable channel.
    fn control_elements(id: AudioObjectId, selector: u32) -> Vec<u32> {
        if settable(id, &address(selector, SCOPE_INPUT, ELEMENT_MAIN)) {
            return vec![ELEMENT_MAIN];
        }
        (1..=MAX_CHANNEL)
            .filter(|&element| settable(id, &address(selector, SCOPE_INPUT, element)))
            .collect()
    }

    /// Mutes or unmutes every input device. Returns how many devices changed.
    pub fn set_muted(muted: bool) -> usize {
        let mut saved = SAVED_VOLUMES.lock().unwrap_or_else(|e| e.into_inner());
        let saved = saved.get_or_insert_with(HashMap::new);
        let mut changed = 0;
        for id in input_devices() {
            let mut ok = false;
            // Every channel must be set, so no short-circuiting `any`.
            for element in control_elements(id, MUTE) {
                ok |= set(id, &address(MUTE, SCOPE_INPUT, element), u32::from(muted));
            }
            for element in control_elements(id, VOLUME) {
                let volume_address = address(VOLUME, SCOPE_INPUT, element);
                let current = get::<f32>(id, &volume_address);
                if muted {
                    // Keep the first saved value: muting again (a device was
                    // plugged in while muted) must not overwrite it with 0.
                    // A device already at 0 is saved as 0 and stays there.
                    if let Some(volume) = current {
                        saved.entry((id, element)).or_insert(volume);
                    }
                    ok |= set(id, &volume_address, 0.0f32);
                } else if let Some(volume) = saved.remove(&(id, element)) {
                    ok |= set(id, &volume_address, volume);
                } else if current.is_some_and(|volume| volume <= SILENT_VOLUME) {
                    ok |= set(id, &volume_address, UNKNOWN_RESTORE_VOLUME);
                }
            }
            changed += usize::from(ok);
        }
        changed
    }

    /// Whether the default input device is muted or at volume 0.
    pub fn is_muted() -> bool {
        let Some(id) = get::<AudioObjectId>(
            SYSTEM_OBJECT,
            &address(DEFAULT_INPUT_DEVICE, SCOPE_GLOBAL, ELEMENT_MAIN),
        )
        .filter(|&id| id != 0) else {
            return false;
        };
        let muted = control_elements(id, MUTE).first().is_some_and(|&element| {
            get::<u32>(id, &address(MUTE, SCOPE_INPUT, element)) == Some(1)
        });
        let silent = control_elements(id, VOLUME)
            .first()
            .and_then(|&element| get::<f32>(id, &address(VOLUME, SCOPE_INPUT, element)))
            .is_some_and(|volume| volume <= SILENT_VOLUME);
        muted || silent
    }
}

/// Mutes or unmutes the microphone system-wide. Returns how many input
/// devices changed (always 0 where unsupported).
pub fn set_muted(muted: bool) -> usize {
    #[cfg(target_os = "macos")]
    {
        macos::set_muted(muted)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = muted;
        0
    }
}

/// Whether the system default microphone is muted.
pub fn is_muted() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::is_muted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
