#[cfg(any(target_os = "macos", target_os = "windows", test))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri_plugin_global_shortcut::ShortcutState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeHotkeyTrigger {
    Fn,
    FnSpace,
    FnLeftShift,
    RightAlt,
    RightAltSpace,
    RightAltLeftShift,
    /// Right Command on macOS, right Windows key on Windows.
    RightCommand,
    RightCtrl,
    RightShift,
    /// A right-hand modifier held with one more key, e.g. RightCommand+/.
    /// RightAlt+Space and RightAlt+LeftShift keep their own variants.
    RightCombo(RightModifier, NativeComboKey),
}

/// Right-hand modifier keys that can act as a native hotkey base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RightModifier {
    Command,
    Alt,
    Ctrl,
    Shift,
}

impl RightModifier {
    pub fn trigger(self) -> NativeHotkeyTrigger {
        match self {
            Self::Command => NativeHotkeyTrigger::RightCommand,
            Self::Alt => NativeHotkeyTrigger::RightAlt,
            Self::Ctrl => NativeHotkeyTrigger::RightCtrl,
            Self::Shift => NativeHotkeyTrigger::RightShift,
        }
    }

    pub fn from_trigger(trigger: NativeHotkeyTrigger) -> Option<Self> {
        match trigger {
            NativeHotkeyTrigger::RightCommand => Some(Self::Command),
            NativeHotkeyTrigger::RightAlt => Some(Self::Alt),
            NativeHotkeyTrigger::RightCtrl => Some(Self::Ctrl),
            NativeHotkeyTrigger::RightShift => Some(Self::Shift),
            _ => None,
        }
    }

    /// Parses a hotkey part such as "RightCommand".
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "rightcommand" => Some(Self::Command),
            "rightalt" | "right_alt" | "right-alt" | "altright" | "alt_right" | "alt-right" => {
                Some(Self::Alt)
            }
            "rightctrl" => Some(Self::Ctrl),
            "rightshift" => Some(Self::Shift),
            _ => None,
        }
    }
}

/// The second key of a native combo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeComboKey {
    Space,
    LeftShift,
    Key(NativeKey),
}

impl NativeComboKey {
    /// Parses the primary part of a hotkey such as "Space", "LeftShift" or "/".
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "space" => Some(Self::Space),
            "leftshift" | "left_shift" | "left-shift" => Some(Self::LeftShift),
            _ => NativeKey::from_name(name).map(Self::Key),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Space => "Space",
            Self::LeftShift => "LeftShift",
            Self::Key(key) => key.name(),
        }
    }
}

/// A regular key usable after a right-hand modifier, with its macOS virtual
/// keycode (ANSI position) and Windows virtual-key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeKey(u8);

struct NativeKeySpec {
    name: &'static str,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    mac: i64,
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    win: u32,
}

const fn key(name: &'static str, mac: i64, win: u32) -> NativeKeySpec {
    NativeKeySpec { name, mac, win }
}

const NATIVE_KEYS: [NativeKeySpec; 67] = [
    key("A", 0, 0x41),
    key("B", 11, 0x42),
    key("C", 8, 0x43),
    key("D", 2, 0x44),
    key("E", 14, 0x45),
    key("F", 3, 0x46),
    key("G", 5, 0x47),
    key("H", 4, 0x48),
    key("I", 34, 0x49),
    key("J", 38, 0x4A),
    key("K", 40, 0x4B),
    key("L", 37, 0x4C),
    key("M", 46, 0x4D),
    key("N", 45, 0x4E),
    key("O", 31, 0x4F),
    key("P", 35, 0x50),
    key("Q", 12, 0x51),
    key("R", 15, 0x52),
    key("S", 1, 0x53),
    key("T", 17, 0x54),
    key("U", 32, 0x55),
    key("V", 9, 0x56),
    key("W", 13, 0x57),
    key("X", 7, 0x58),
    key("Y", 16, 0x59),
    key("Z", 6, 0x5A),
    key("0", 29, 0x30),
    key("1", 18, 0x31),
    key("2", 19, 0x32),
    key("3", 20, 0x33),
    key("4", 21, 0x34),
    key("5", 23, 0x35),
    key("6", 22, 0x36),
    key("7", 26, 0x37),
    key("8", 28, 0x38),
    key("9", 25, 0x39),
    key("/", 44, 0xBF),
    key(".", 47, 0xBE),
    key(",", 43, 0xBC),
    key(";", 41, 0xBA),
    key("'", 39, 0xDE),
    key("[", 33, 0xDB),
    key("]", 30, 0xDD),
    key("-", 27, 0xBD),
    key("=", 24, 0xBB),
    key("`", 50, 0xC0),
    key("\\", 42, 0xDC),
    key("Enter", 36, 0x0D),
    key("Tab", 48, 0x09),
    key("Escape", 53, 0x1B),
    key("Backspace", 51, 0x08),
    key("Up", 126, 0x26),
    key("Down", 125, 0x28),
    key("Left", 123, 0x25),
    key("Right", 124, 0x27),
    key("F1", 122, 0x70),
    key("F2", 120, 0x71),
    key("F3", 99, 0x72),
    key("F4", 118, 0x73),
    key("F5", 96, 0x74),
    key("F6", 97, 0x75),
    key("F7", 98, 0x76),
    key("F8", 100, 0x77),
    key("F9", 101, 0x78),
    key("F10", 109, 0x79),
    key("F11", 103, 0x7A),
    key("F12", 111, 0x7B),
];

impl NativeKey {
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim();
        NATIVE_KEYS
            .iter()
            .position(|spec| spec.name.eq_ignore_ascii_case(name))
            .map(|index| Self(index as u8))
    }

    pub fn name(self) -> &'static str {
        NATIVE_KEYS[self.0 as usize].name
    }

    #[cfg(target_os = "macos")]
    fn from_mac_keycode(keycode: i64) -> Option<Self> {
        NATIVE_KEYS
            .iter()
            .position(|spec| spec.mac == keycode)
            .map(|index| Self(index as u8))
    }

    #[cfg(target_os = "windows")]
    fn from_vk(vk: u32) -> Option<Self> {
        NATIVE_KEYS
            .iter()
            .position(|spec| spec.win == vk)
            .map(|index| Self(index as u8))
    }
}

impl NativeHotkeyTrigger {
    pub fn canonical(self) -> String {
        match self {
            Self::Fn => "Fn".to_string(),
            Self::FnSpace => "Fn+Space".to_string(),
            Self::FnLeftShift => "Fn+LeftShift".to_string(),
            Self::RightAlt => "RightAlt".to_string(),
            Self::RightAltSpace => "RightAlt+Space".to_string(),
            Self::RightAltLeftShift => "RightAlt+LeftShift".to_string(),
            Self::RightCommand => "RightCommand".to_string(),
            Self::RightCtrl => "RightCtrl".to_string(),
            Self::RightShift => "RightShift".to_string(),
            Self::RightCombo(modifier, combo) => {
                format!("{}+{}", modifier.trigger().canonical(), combo.name())
            }
        }
    }

    /// Bare right-hand modifiers. Every trigger in this list is its own base.
    #[cfg(target_os = "windows")]
    const BARE_MODIFIERS: [NativeHotkeyTrigger; 4] = [
        Self::RightAlt,
        Self::RightCommand,
        Self::RightCtrl,
        Self::RightShift,
    ];

    #[cfg(any(target_os = "macos", target_os = "windows", test))]
    fn base(self) -> NativeHotkeyTrigger {
        match self {
            Self::Fn | Self::FnSpace | Self::FnLeftShift => Self::Fn,
            Self::RightAlt | Self::RightAltSpace | Self::RightAltLeftShift => Self::RightAlt,
            Self::RightCommand => Self::RightCommand,
            Self::RightCtrl => Self::RightCtrl,
            Self::RightShift => Self::RightShift,
            Self::RightCombo(modifier, _) => modifier.trigger(),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows", test))]
    fn combo_key(self) -> Option<NativeComboKey> {
        match self {
            Self::FnSpace | Self::RightAltSpace => Some(NativeComboKey::Space),
            Self::FnLeftShift | Self::RightAltLeftShift => Some(NativeComboKey::LeftShift),
            Self::RightCombo(_, combo) => Some(combo),
            Self::Fn | Self::RightAlt | Self::RightCommand | Self::RightCtrl | Self::RightShift => {
                None
            }
        }
    }

    pub fn from_base_combo(base: NativeHotkeyTrigger, combo: NativeComboKey) -> Option<Self> {
        match (base, combo) {
            (Self::Fn, NativeComboKey::Space) => Some(Self::FnSpace),
            (Self::Fn, NativeComboKey::LeftShift) => Some(Self::FnLeftShift),
            (Self::RightAlt, NativeComboKey::Space) => Some(Self::RightAltSpace),
            (Self::RightAlt, NativeComboKey::LeftShift) => Some(Self::RightAltLeftShift),
            (Self::RightShift, NativeComboKey::LeftShift) => None,
            (base, combo) => {
                RightModifier::from_trigger(base).map(|modifier| Self::RightCombo(modifier, combo))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeHotkeyBinding {
    pub role: crate::hotkey::HotkeyRole,
    pub index: usize,
    pub trigger: NativeHotkeyTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeHotkeyEvent {
    pub role: crate::hotkey::HotkeyRole,
    pub index: usize,
    pub state: ShortcutState,
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
#[derive(Default)]
struct NativeHeldState {
    held: AtomicBool,
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
impl NativeHeldState {
    fn edge(&self, pressed: bool) -> Option<ShortcutState> {
        match (pressed, self.held.swap(pressed, Ordering::SeqCst)) {
            (true, false) => Some(ShortcutState::Pressed),
            (false, true) => Some(ShortcutState::Released),
            _ => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct NativeHotkeyRuntime {
    inner: Arc<Mutex<NativeHotkeyRuntimeInner>>,
}

#[derive(Default)]
struct NativeHotkeyRuntimeInner {
    monitor: Option<platform::PlatformNativeMonitor>,
}

impl NativeHotkeyRuntime {
    pub fn install(
        &self,
        bindings: Vec<NativeHotkeyBinding>,
        hold_dictation: bool,
        handler: Arc<dyn Fn(NativeHotkeyEvent) + Send + Sync>,
    ) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let _ = inner.monitor.take();

        if bindings.is_empty() {
            return Ok(());
        }

        inner.monitor = Some(platform::PlatformNativeMonitor::start(
            bindings,
            hold_dictation,
            handler,
        )?);
        Ok(())
    }

    pub fn pause(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let _ = inner.monitor.take();
    }
}

type NativeHotkeyHandler = Arc<dyn Fn(NativeHotkeyEvent) + Send + Sync + 'static>;

#[cfg(any(target_os = "macos", target_os = "windows", test))]
struct NativeMonitoredBinding {
    binding: NativeHotkeyBinding,
    held: NativeHeldState,
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
impl NativeMonitoredBinding {
    fn new(binding: NativeHotkeyBinding) -> Self {
        Self {
            binding,
            held: NativeHeldState::default(),
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn monitored_bindings_for_bases(
    bindings: Vec<NativeHotkeyBinding>,
    bases: &[NativeHotkeyTrigger],
) -> Vec<NativeMonitoredBinding> {
    bindings
        .into_iter()
        .filter(|binding| bases.contains(&binding.trigger.base()))
        .map(NativeMonitoredBinding::new)
        .collect()
}

/// Bases that have at least one binding, in a stable order.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn bound_bases(bindings: &[NativeMonitoredBinding]) -> Vec<NativeHotkeyTrigger> {
    let mut bases = Vec::new();
    for binding in bindings {
        let base = binding.binding.trigger.base();
        if !bases.contains(&base) {
            bases.push(base);
        }
    }
    bases
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn has_combo_bindings(bindings: &[NativeMonitoredBinding], base: NativeHotkeyTrigger) -> bool {
    bindings.iter().any(|binding| {
        binding.binding.trigger.base() == base && binding.binding.trigger.combo_key().is_some()
    })
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn has_binding_for_trigger(
    bindings: &[NativeMonitoredBinding],
    trigger: NativeHotkeyTrigger,
) -> bool {
    bindings
        .iter()
        .any(|binding| binding.binding.trigger == trigger)
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn dispatch_matching_bindings(
    bindings: &[NativeMonitoredBinding],
    trigger: NativeHotkeyTrigger,
    pressed: bool,
    handler: &NativeHotkeyHandler,
) -> bool {
    let mut matched = false;
    for binding in bindings {
        if binding.binding.trigger != trigger {
            continue;
        }
        matched = true;
        if let Some(state) = binding.held.edge(pressed) {
            (handler.as_ref())(NativeHotkeyEvent {
                role: binding.binding.role,
                index: binding.binding.index,
                state,
            });
        }
    }
    matched
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
#[derive(Default)]
struct NativeComboState {
    base_pressed: bool,
    pending_base_press: bool,
    combo_used: bool,
    hold_base: bool,
    /// Wait for the release (or the hold grace period) before firing a bare
    /// base, so the key still works as a modifier: Right Command + C must not
    /// start dictation.
    defer_bare: bool,
    edge_generation: u64,
}

#[cfg(any(target_os = "macos", test))]
fn activate_pending_hold_base(
    state: &mut NativeComboState,
    bindings: &[NativeMonitoredBinding],
    base: NativeHotkeyTrigger,
    edge_generation: u64,
    handler: &NativeHotkeyHandler,
) -> bool {
    if !state.hold_base
        || !state.base_pressed
        || !state.pending_base_press
        || state.combo_used
        || state.edge_generation != edge_generation
    {
        return false;
    }
    state.pending_base_press = false;
    dispatch_matching_bindings(bindings, base, true, handler)
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn dispatch_native_base_edge(
    state: &mut NativeComboState,
    bindings: &[NativeMonitoredBinding],
    base: NativeHotkeyTrigger,
    pressed: bool,
    handler: &NativeHotkeyHandler,
) -> bool {
    if state.base_pressed == pressed {
        return false;
    }
    state.edge_generation = state.edge_generation.wrapping_add(1);
    if pressed {
        state.base_pressed = true;
        state.combo_used = false;
        state.pending_base_press = state.defer_bare || has_combo_bindings(bindings, base);
        if state.pending_base_press {
            return true;
        }
        return dispatch_matching_bindings(bindings, base, true, handler);
    }

    if state.pending_base_press && !state.combo_used {
        if state.hold_base {
            state.base_pressed = false;
            state.pending_base_press = false;
            return true;
        }
        let matched = dispatch_matching_bindings(bindings, base, true, handler);
        let _ = dispatch_matching_bindings(bindings, base, false, handler);
        state.base_pressed = false;
        state.pending_base_press = false;
        return matched;
    }

    if state.combo_used {
        let combo_triggers: Vec<NativeHotkeyTrigger> = bindings
            .iter()
            .map(|binding| binding.binding.trigger)
            .filter(|trigger| trigger.base() == base && trigger.combo_key().is_some())
            .collect();
        for trigger in combo_triggers {
            let _ = dispatch_matching_bindings(bindings, trigger, false, handler);
        }
        state.base_pressed = false;
        state.pending_base_press = false;
        state.combo_used = false;
        return true;
    }

    state.base_pressed = false;
    dispatch_matching_bindings(bindings, base, false, handler)
}

/// Another key went down while a deferred base is held: the user is typing a
/// regular shortcut (e.g. Right Command + C), so the bare base must not fire.
/// An already active hold is stopped.
#[cfg(any(target_os = "macos", test))]
fn cancel_native_base_on_other_key(
    state: &mut NativeComboState,
    bindings: &[NativeMonitoredBinding],
    base: NativeHotkeyTrigger,
    handler: &NativeHotkeyHandler,
) {
    if !state.defer_bare || !state.base_pressed || state.combo_used {
        return;
    }
    let active = !state.pending_base_press;
    state.combo_used = true;
    state.pending_base_press = false;
    state.edge_generation = state.edge_generation.wrapping_add(1);
    if active {
        let _ = dispatch_matching_bindings(bindings, base, false, handler);
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn dispatch_native_combo_edge(
    state: &mut NativeComboState,
    bindings: &[NativeMonitoredBinding],
    base: NativeHotkeyTrigger,
    combo: NativeComboKey,
    pressed: bool,
    handler: &NativeHotkeyHandler,
) -> bool {
    if !state.base_pressed {
        return false;
    }
    let Some(trigger) = NativeHotkeyTrigger::from_base_combo(base, combo) else {
        return false;
    };
    if !has_binding_for_trigger(bindings, trigger) {
        return false;
    }

    if pressed {
        if state.hold_base {
            // A slowly entered combo may follow an already active hold.
            // Stop the bare shortcut before dispatching the combo.
            let _ = dispatch_matching_bindings(bindings, base, false, handler);
        }
        state.combo_used = true;
        state.pending_base_press = false;
    }
    dispatch_matching_bindings(bindings, trigger, pressed, handler)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{
        activate_pending_hold_base, bound_bases, cancel_native_base_on_other_key,
        dispatch_native_base_edge, dispatch_native_combo_edge, monitored_bindings_for_bases,
        NativeComboKey, NativeComboState, NativeHotkeyHandler, NativeHotkeyTrigger, NativeKey,
        NativeMonitoredBinding,
    };
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    const STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
    const HOLD_COMBO_GRACE_PERIOD: Duration = Duration::from_millis(200);

    type CgEventMask = u64;
    type CgEventType = u32;
    type CgEventTapLocation = u32;
    type CgEventTapPlacement = u32;
    type CgEventTapOptions = u32;
    type CgEventField = u32;
    type CgEventFlags = u64;
    type CfStringRef = *const c_void;
    type CfAllocatorRef = *const c_void;

    #[repr(C)]
    struct OpaqueCgEvent(c_void);
    type CgEventRef = *mut OpaqueCgEvent;

    #[repr(C)]
    struct OpaqueCfMachPort(c_void);
    type CfMachPortRef = *mut OpaqueCfMachPort;

    #[repr(C)]
    struct OpaqueCfRunLoop(c_void);
    type CfRunLoopRef = *mut OpaqueCfRunLoop;

    #[repr(C)]
    struct OpaqueCfRunLoopSource(c_void);
    type CfRunLoopSourceRef = *mut OpaqueCfRunLoopSource;

    const SESSION_EVENT_TAP: CgEventTapLocation = 1;
    const HEAD_INSERT: CgEventTapPlacement = 0;
    const TAP_OPTION_DEFAULT: CgEventTapOptions = 0;

    const KEY_DOWN: CgEventType = 10;
    const KEY_UP: CgEventType = 11;
    const FLAGS_CHANGED: CgEventType = 12;
    const TAP_DISABLED_BY_TIMEOUT: CgEventType = 0xFFFF_FFFE;
    const TAP_DISABLED_BY_USER_INPUT: CgEventType = 0xFFFF_FFFF;

    const KEYBOARD_EVENT_KEYCODE: CgEventField = 9;
    const FLAG_MASK_SECONDARY_FN: CgEventFlags = 0x0080_0000;
    const FLAG_MASK_SHIFT: CgEventFlags = 0x0002_0000;
    // Device-dependent flag bits (IOLLEvent.h NX_DEVICER*KEYMASK) tell the
    // right-hand modifier apart from the left one.
    const FLAG_MASK_DEVICE_RIGHT_SHIFT: CgEventFlags = 0x0000_0004;
    const FLAG_MASK_DEVICE_RIGHT_COMMAND: CgEventFlags = 0x0000_0010;
    const FLAG_MASK_DEVICE_RIGHT_OPTION: CgEventFlags = 0x0000_0040;
    const FLAG_MASK_DEVICE_RIGHT_CONTROL: CgEventFlags = 0x0000_2000;
    const FN_KEYCODE: i64 = 63;
    const SPACE_KEYCODE: i64 = 49;
    const LEFT_SHIFT_KEYCODE: i64 = 56;
    const RIGHT_COMMAND_KEYCODE: i64 = 54;
    const RIGHT_SHIFT_KEYCODE: i64 = 60;
    const RIGHT_OPTION_KEYCODE: i64 = 61;
    const RIGHT_CONTROL_KEYCODE: i64 = 62;

    const MAC_BASES: [NativeHotkeyTrigger; 5] = [
        NativeHotkeyTrigger::Fn,
        NativeHotkeyTrigger::RightAlt,
        NativeHotkeyTrigger::RightCommand,
        NativeHotkeyTrigger::RightCtrl,
        NativeHotkeyTrigger::RightShift,
    ];

    type CgEventTapCallBack = extern "C" fn(
        proxy: *mut c_void,
        event_type: CgEventType,
        event: CgEventRef,
        user_info: *mut c_void,
    ) -> CgEventRef;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventTapCreate(
            tap: CgEventTapLocation,
            place: CgEventTapPlacement,
            options: CgEventTapOptions,
            events_of_interest: CgEventMask,
            callback: CgEventTapCallBack,
            user_info: *mut c_void,
        ) -> CfMachPortRef;
        fn CGEventTapEnable(tap: CfMachPortRef, enable: bool);
        fn CGEventGetIntegerValueField(event: CgEventRef, field: CgEventField) -> i64;
        fn CGEventGetFlags(event: CgEventRef) -> CgEventFlags;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFMachPortCreateRunLoopSource(
            allocator: CfAllocatorRef,
            port: CfMachPortRef,
            order: isize,
        ) -> CfRunLoopSourceRef;
        fn CFRunLoopGetCurrent() -> CfRunLoopRef;
        fn CFRunLoopAddSource(rl: CfRunLoopRef, source: CfRunLoopSourceRef, mode: CfStringRef);
        fn CFRunLoopRun();
        fn CFRunLoopStop(rl: CfRunLoopRef);
        fn CFRelease(cf: *const c_void);
        static kCFRunLoopCommonModes: CfStringRef;
    }

    struct MacShutdownHandles {
        tap: Mutex<Option<CfMachPortRef>>,
        runloop: Mutex<Option<CfRunLoopRef>>,
        cancelled: AtomicBool,
    }

    unsafe impl Send for MacShutdownHandles {}
    unsafe impl Sync for MacShutdownHandles {}

    impl MacShutdownHandles {
        fn new() -> Self {
            Self {
                tap: Mutex::new(None),
                runloop: Mutex::new(None),
                cancelled: AtomicBool::new(false),
            }
        }

        fn shutdown(&self) {
            self.cancelled.store(true, Ordering::SeqCst);
            if let Some(tap) = self.tap.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                unsafe { CGEventTapEnable(*tap, false) };
            }
            if let Some(runloop) = self
                .runloop
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
            {
                unsafe { CFRunLoopStop(*runloop) };
            }
        }
    }

    pub struct PlatformNativeMonitor {
        handles: Arc<MacShutdownHandles>,
    }

    impl PlatformNativeMonitor {
        pub fn start(
            bindings: Vec<super::NativeHotkeyBinding>,
            hold_dictation: bool,
            handler: NativeHotkeyHandler,
        ) -> Result<Self, String> {
            let bindings = monitored_bindings_for_bases(bindings, &MAC_BASES);
            if bindings.is_empty() {
                return Err(
                    "macOS native hotkeys support Fn and right-hand modifier keys only".to_string(),
                );
            }

            let handles = Arc::new(MacShutdownHandles::new());
            let thread_handles = Arc::clone(&handles);
            let (status_tx, status_rx) = mpsc::channel();
            thread::Builder::new()
                .name("talkey-native-hotkey-mac".to_string())
                .spawn(move || {
                    run_event_tap_loop(bindings, hold_dictation, handler, thread_handles, status_tx)
                })
                .map_err(|error| {
                    format!("Failed to spawn macOS native hotkey monitor thread: {error}")
                })?;

            match status_rx.recv_timeout(STARTUP_TIMEOUT) {
                Ok(Ok(())) => Ok(Self { handles }),
                Ok(Err(error)) => {
                    handles.shutdown();
                    Err(error)
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    handles.shutdown();
                    Err(
                        "Timed out starting macOS native hotkey EventTap after 3 seconds"
                            .to_string(),
                    )
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    handles.shutdown();
                    Err(
                        "macOS native hotkey EventTap thread exited before startup completed"
                            .to_string(),
                    )
                }
            }
        }
    }

    impl Drop for PlatformNativeMonitor {
        fn drop(&mut self) {
            self.handles.shutdown();
        }
    }

    type SharedComboState = Arc<Mutex<NativeComboState>>;

    struct CallbackContext {
        bindings: Arc<Vec<NativeMonitoredBinding>>,
        handler: NativeHotkeyHandler,
        handles: Arc<MacShutdownHandles>,
        /// One combo state per bound base key (Fn, Right Command, ...).
        states: Vec<(NativeHotkeyTrigger, SharedComboState)>,
    }

    impl CallbackContext {
        fn state_for(&self, base: NativeHotkeyTrigger) -> Option<&SharedComboState> {
            self.states
                .iter()
                .find(|(candidate, _)| *candidate == base)
                .map(|(_, state)| state)
        }

        /// A key other than `except` went down: deferred bases that are being
        /// held become part of a regular shortcut and must not fire.
        fn cancel_deferred_bases(&self, except: Option<NativeHotkeyTrigger>) {
            for (base, state) in &self.states {
                if Some(*base) == except {
                    continue;
                }
                let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
                cancel_native_base_on_other_key(&mut state, &self.bindings, *base, &self.handler);
            }
        }
    }

    fn run_event_tap_loop(
        bindings: Vec<NativeMonitoredBinding>,
        hold_dictation: bool,
        handler: NativeHotkeyHandler,
        handles: Arc<MacShutdownHandles>,
        status_tx: mpsc::Sender<Result<(), String>>,
    ) {
        let states = bound_bases(&bindings)
            .into_iter()
            .map(|base| {
                let hold_base = hold_dictation
                    && bindings.iter().any(|binding| {
                        binding.binding.trigger == base
                            && binding.binding.role == crate::hotkey::HotkeyRole::Dictation
                    });
                let state = NativeComboState {
                    hold_base,
                    // Right-hand modifiers keep working as modifiers; only a
                    // lone press triggers the hotkey.
                    defer_bare: base != NativeHotkeyTrigger::Fn,
                    ..NativeComboState::default()
                };
                (base, Arc::new(Mutex::new(state)))
            })
            .collect();
        let context = Box::into_raw(Box::new(CallbackContext {
            bindings: Arc::new(bindings),
            handler,
            handles: Arc::clone(&handles),
            states,
        }));
        let mask: CgEventMask = (1u64 << FLAGS_CHANGED) | (1u64 << KEY_DOWN) | (1u64 << KEY_UP);

        unsafe {
            let tap = CGEventTapCreate(
                SESSION_EVENT_TAP,
                HEAD_INSERT,
                TAP_OPTION_DEFAULT,
                mask,
                event_tap_callback,
                context as *mut c_void,
            );
            if tap.is_null() {
                drop(Box::from_raw(context));
                let _ = status_tx.send(Err(
                    "Failed to create macOS native hotkey EventTap; Accessibility permission may be denied"
                        .to_string(),
                ));
                return;
            }
            *handles.tap.lock().unwrap_or_else(|e| e.into_inner()) = Some(tap);

            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
            if source.is_null() {
                CGEventTapEnable(tap, false);
                *handles.tap.lock().unwrap_or_else(|e| e.into_inner()) = None;
                CFRelease(tap as *const c_void);
                drop(Box::from_raw(context));
                let _ = status_tx.send(Err(
                    "Failed to create macOS native hotkey EventTap run loop source".to_string(),
                ));
                return;
            }

            let runloop = CFRunLoopGetCurrent();
            *handles.runloop.lock().unwrap_or_else(|e| e.into_inner()) = Some(runloop);

            if handles.cancelled.load(Ordering::SeqCst) {
                CGEventTapEnable(tap, false);
                CFRelease(source as *const c_void);
                *handles.tap.lock().unwrap_or_else(|e| e.into_inner()) = None;
                *handles.runloop.lock().unwrap_or_else(|e| e.into_inner()) = None;
                CFRelease(tap as *const c_void);
                drop(Box::from_raw(context));
                let _ = status_tx.send(Err(
                    "macOS native hotkey EventTap startup was cancelled".to_string()
                ));
                return;
            }

            CFRunLoopAddSource(runloop, source, kCFRunLoopCommonModes);
            CFRelease(source as *const c_void);
            CGEventTapEnable(tap, true);

            if status_tx.send(Ok(())).is_err() || handles.cancelled.load(Ordering::SeqCst) {
                handles.shutdown();
            }

            if !handles.cancelled.load(Ordering::SeqCst) {
                CFRunLoopRun();
            }

            if let Some(tap) = handles.tap.lock().unwrap_or_else(|e| e.into_inner()).take() {
                CGEventTapEnable(tap, false);
                CFRelease(tap as *const c_void);
            }
            *handles.runloop.lock().unwrap_or_else(|e| e.into_inner()) = None;
            drop(Box::from_raw(context));
        }
    }

    extern "C" fn event_tap_callback(
        _proxy: *mut c_void,
        event_type: CgEventType,
        event: CgEventRef,
        user_info: *mut c_void,
    ) -> CgEventRef {
        if user_info.is_null() {
            return event;
        }
        let context = unsafe { &*(user_info as *const CallbackContext) };

        match event_type {
            TAP_DISABLED_BY_TIMEOUT | TAP_DISABLED_BY_USER_INPUT => {
                // macOS can disable the tap while a key is held. Clear the
                // captured state so a missing key-up cannot leave a key stuck.
                for (base, state) in &context.states {
                    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = dispatch_native_base_edge(
                        &mut state,
                        &context.bindings,
                        *base,
                        false,
                        &context.handler,
                    );
                }
                if let Some(tap) = context
                    .handles
                    .tap
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .as_ref()
                {
                    unsafe { CGEventTapEnable(*tap, true) };
                }
            }
            FLAGS_CHANGED => handle_flags_changed(context, event),
            // A right-hand modifier combo matched: keep the key away from
            // the focused app (Right Command + / is not Cmd+/).
            KEY_DOWN | KEY_UP if handle_key_event(context, event, event_type == KEY_DOWN) => {
                return std::ptr::null_mut();
            }
            _ => {}
        }

        event
    }

    /// Feeds a combo key edge to every bound base. Returns true when a combo
    /// on a right-hand modifier matched, so the key event should be dropped.
    fn dispatch_combo_to_bases(
        context: &CallbackContext,
        combo: NativeComboKey,
        pressed: bool,
    ) -> (bool, bool) {
        let mut matched = false;
        let mut swallow = false;
        for (base, state) in &context.states {
            let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
            if dispatch_native_combo_edge(
                &mut state,
                &context.bindings,
                *base,
                combo,
                pressed,
                &context.handler,
            ) {
                matched = true;
                swallow |= state.defer_bare;
            }
        }
        (matched, swallow)
    }

    fn handle_flags_changed(context: &CallbackContext, event: CgEventRef) {
        let keycode = unsafe { CGEventGetIntegerValueField(event, KEYBOARD_EVENT_KEYCODE) };
        let flags = unsafe { CGEventGetFlags(event) };
        let base_edge = match keycode {
            FN_KEYCODE => Some((NativeHotkeyTrigger::Fn, FLAG_MASK_SECONDARY_FN)),
            RIGHT_COMMAND_KEYCODE => Some((
                NativeHotkeyTrigger::RightCommand,
                FLAG_MASK_DEVICE_RIGHT_COMMAND,
            )),
            RIGHT_OPTION_KEYCODE => {
                Some((NativeHotkeyTrigger::RightAlt, FLAG_MASK_DEVICE_RIGHT_OPTION))
            }
            RIGHT_CONTROL_KEYCODE => Some((
                NativeHotkeyTrigger::RightCtrl,
                FLAG_MASK_DEVICE_RIGHT_CONTROL,
            )),
            RIGHT_SHIFT_KEYCODE => Some((
                NativeHotkeyTrigger::RightShift,
                FLAG_MASK_DEVICE_RIGHT_SHIFT,
            )),
            _ => None,
        };

        if let Some((base, mask)) = base_edge {
            let pressed = (flags & mask) != 0;
            if pressed {
                context.cancel_deferred_bases(Some(base));
            }
            if let Some(state) = context.state_for(base) {
                handle_base_edge(context, base, state, pressed);
            }
            return;
        }

        if keycode == LEFT_SHIFT_KEYCODE {
            let pressed = (flags & FLAG_MASK_SHIFT) != 0;
            let (matched, _) = dispatch_combo_to_bases(context, NativeComboKey::LeftShift, pressed);
            if matched {
                return;
            }
        }

        // Any other modifier change (left Command, Caps Lock, ...) while a
        // right-hand modifier is held means a regular shortcut.
        context.cancel_deferred_bases(None);
    }

    fn handle_base_edge(
        context: &CallbackContext,
        base: NativeHotkeyTrigger,
        shared_state: &SharedComboState,
        pressed: bool,
    ) {
        let mut state = shared_state.lock().unwrap_or_else(|e| e.into_inner());
        let was_pressed = state.base_pressed;
        let _ = dispatch_native_base_edge(
            &mut state,
            &context.bindings,
            base,
            pressed,
            &context.handler,
        );
        if pressed && !was_pressed && state.hold_base && state.pending_base_press {
            let generation = state.edge_generation;
            let state = Arc::clone(shared_state);
            let bindings = Arc::clone(&context.bindings);
            let handler = Arc::clone(&context.handler);
            let handles = Arc::clone(&context.handles);
            if let Err(error) = thread::Builder::new()
                .name("talkey-hotkey-hold-delay".to_string())
                .spawn(move || {
                    thread::sleep(HOLD_COMBO_GRACE_PERIOD);
                    if handles.cancelled.load(Ordering::SeqCst) {
                        return;
                    }
                    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = activate_pending_hold_base(
                        &mut state, &bindings, base, generation, &handler,
                    );
                })
            {
                tracing::warn!(
                    "Failed to schedule {} hold hotkey: {error}",
                    base.canonical()
                );
            }
        }
    }

    /// Returns true when the key event belongs to a right-hand modifier combo
    /// and must not reach the focused app.
    fn handle_key_event(context: &CallbackContext, event: CgEventRef, pressed: bool) -> bool {
        let keycode = unsafe { CGEventGetIntegerValueField(event, KEYBOARD_EVENT_KEYCODE) };
        let combo = if keycode == SPACE_KEYCODE {
            Some(NativeComboKey::Space)
        } else {
            NativeKey::from_mac_keycode(keycode).map(NativeComboKey::Key)
        };
        let (matched, swallow) = match combo {
            Some(combo) => dispatch_combo_to_bases(context, combo, pressed),
            None => (false, false),
        };
        if pressed && !matched {
            context.cancel_deferred_bases(None);
        }
        swallow
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::{
        bound_bases, dispatch_native_base_edge, dispatch_native_combo_edge,
        monitored_bindings_for_bases, NativeComboKey, NativeComboState, NativeHotkeyHandler,
        NativeHotkeyTrigger, NativeKey, NativeMonitoredBinding,
    };
    use std::ptr;
    use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering as AtomicOrdering};
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{GetLastError, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, PeekMessageW, PostThreadMessageW,
        SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HC_ACTION, HHOOK,
        KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WM_QUIT,
    };

    const STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
    const ACCEPT_SYNTHETIC_EVENTS_ENV: &str = "OPENTYPELESS_ACCEPT_SYNTHETIC_HOTKEY_EVENTS";

    const WM_KEYDOWN: usize = 0x0100;
    const WM_KEYUP: usize = 0x0101;
    const WM_SYSKEYDOWN: usize = 0x0104;
    const WM_SYSKEYUP: usize = 0x0105;
    const VK_SPACE: u32 = 0x20;
    const VK_LSHIFT: u32 = 0xA0;
    const VK_RSHIFT: u32 = 0xA1;
    const VK_RCONTROL: u32 = 0xA3;
    const VK_RMENU: u32 = 0xA5;
    const VK_RWIN: u32 = 0x5C;
    const VK_HANGUL: u32 = 0x15;
    const VK_HANJA: u32 = 0x19;
    const SCAN_CONTROL: u32 = 0x1D;
    const SCAN_ALT: u32 = 0x38;

    static HOOK_CONTEXT: AtomicPtr<CallbackContext> = AtomicPtr::new(ptr::null_mut());

    pub struct PlatformNativeMonitor {
        thread_id: u32,
        thread: Option<thread::JoinHandle<()>>,
    }

    struct WindowsStartupState {
        thread_id: AtomicU32,
        cancelled: AtomicBool,
    }

    impl WindowsStartupState {
        fn new() -> Self {
            Self {
                thread_id: AtomicU32::new(0),
                cancelled: AtomicBool::new(false),
            }
        }

        fn cancel(&self) {
            self.cancelled.store(true, AtomicOrdering::SeqCst);
            let thread_id = self.thread_id.load(AtomicOrdering::SeqCst);
            if thread_id != 0 {
                let ok = unsafe { PostThreadMessageW(thread_id, WM_QUIT, 0, 0) };
                if ok == 0 {
                    tracing::warn!(
                        "Failed to post WM_QUIT to cancelled Windows native hotkey hook thread: {}",
                        unsafe { GetLastError() }
                    );
                }
            }
        }

        fn is_cancelled(&self) -> bool {
            self.cancelled.load(AtomicOrdering::SeqCst)
        }
    }

    impl PlatformNativeMonitor {
        pub fn start(
            bindings: Vec<super::NativeHotkeyBinding>,
            _hold_dictation: bool,
            handler: NativeHotkeyHandler,
        ) -> Result<Self, String> {
            let bindings =
                monitored_bindings_for_bases(bindings, &NativeHotkeyTrigger::BARE_MODIFIERS);
            if bindings.is_empty() {
                return Err(
                    "Windows native hotkeys support right-hand modifier keys only".to_string(),
                );
            }

            let (status_tx, status_rx) = mpsc::channel();
            let startup = Arc::new(WindowsStartupState::new());
            let thread_startup = Arc::clone(&startup);
            let thread = thread::Builder::new()
                .name("talkey-native-hotkey-win".to_string())
                .spawn(move || run_keyboard_hook_loop(bindings, handler, thread_startup, status_tx))
                .map_err(|error| {
                    format!("Failed to spawn Windows native hotkey monitor thread: {error}")
                })?;

            match status_rx.recv_timeout(STARTUP_TIMEOUT) {
                Ok(Ok(thread_id)) => Ok(Self {
                    thread_id,
                    thread: Some(thread),
                }),
                Ok(Err(error)) => {
                    startup.cancel();
                    let _ = thread.join();
                    Err(error)
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    startup.cancel();
                    let _ = thread.join();
                    Err("Timed out starting Windows native hotkey hook after 3 seconds".to_string())
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    startup.cancel();
                    let _ = thread.join();
                    Err(
                        "Windows native hotkey hook thread exited before startup completed"
                            .to_string(),
                    )
                }
            }
        }
    }

    impl Drop for PlatformNativeMonitor {
        fn drop(&mut self) {
            let ok = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0) };
            if ok == 0 {
                tracing::warn!(
                    "Failed to post WM_QUIT to Windows native hotkey hook thread: {}",
                    unsafe { GetLastError() }
                );
            }
            if let Some(thread) = self.thread.take() {
                if thread.thread().id() != thread::current().id() && thread.join().is_err() {
                    tracing::warn!("Windows native hotkey hook thread panicked during shutdown");
                }
            }
        }
    }

    struct CallbackContext {
        bindings: Vec<NativeMonitoredBinding>,
        handler: NativeHotkeyHandler,
        hook: std::sync::Mutex<Option<HHOOK>>,
        /// One combo state per bound base key (RightAlt, RightCtrl, ...).
        states: Vec<(NativeHotkeyTrigger, std::sync::Mutex<NativeComboState>)>,
    }

    impl CallbackContext {
        fn state_for(
            &self,
            base: NativeHotkeyTrigger,
        ) -> Option<&std::sync::Mutex<NativeComboState>> {
            self.states
                .iter()
                .find(|(candidate, _)| *candidate == base)
                .map(|(_, state)| state)
        }
    }

    unsafe impl Send for CallbackContext {}
    unsafe impl Sync for CallbackContext {}

    fn run_keyboard_hook_loop(
        bindings: Vec<NativeMonitoredBinding>,
        handler: NativeHotkeyHandler,
        startup: Arc<WindowsStartupState>,
        status_tx: mpsc::Sender<Result<u32, String>>,
    ) {
        let thread_id = unsafe { GetCurrentThreadId() };

        unsafe {
            let mut message: MSG = std::mem::zeroed();
            let _ = PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_NOREMOVE);
            startup.thread_id.store(thread_id, AtomicOrdering::SeqCst);

            if startup.is_cancelled() {
                let _ = status_tx.send(Err(
                    "Windows native hotkey hook startup was cancelled before install".to_string(),
                ));
                return;
            }

            let states = bound_bases(&bindings)
                .into_iter()
                .map(|base| (base, std::sync::Mutex::new(NativeComboState::default())))
                .collect();
            let context = Box::into_raw(Box::new(CallbackContext {
                bindings,
                handler,
                hook: std::sync::Mutex::new(None),
                states,
            }));
            HOOK_CONTEXT.store(context, AtomicOrdering::SeqCst);

            // Global low-level hooks should be installed with the current module handle.
            let module = GetModuleHandleW(ptr::null());
            if module.is_null() {
                let error = GetLastError();
                let _ = HOOK_CONTEXT.compare_exchange(
                    context,
                    ptr::null_mut(),
                    AtomicOrdering::SeqCst,
                    AtomicOrdering::SeqCst,
                );
                drop(Box::from_raw(context));
                let _ = status_tx.send(Err(format!(
                    "Failed to resolve Windows module handle for native hotkey hook: Win32 error {error}"
                )));
                return;
            }

            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), module, 0);
            if hook.is_null() {
                let _ = HOOK_CONTEXT.compare_exchange(
                    context,
                    ptr::null_mut(),
                    AtomicOrdering::SeqCst,
                    AtomicOrdering::SeqCst,
                );
                drop(Box::from_raw(context));
                let _ = status_tx.send(Err(format!(
                    "Failed to install Windows native hotkey keyboard hook: Win32 error {}",
                    GetLastError()
                )));
                return;
            }
            *(*context).hook.lock().unwrap_or_else(|e| e.into_inner()) = Some(hook);

            if startup.is_cancelled() {
                cleanup_context(context);
                let _ = status_tx.send(Err(
                    "Windows native hotkey hook startup was cancelled after install".to_string(),
                ));
                return;
            }

            if status_tx.send(Ok(thread_id)).is_err() || startup.is_cancelled() {
                cleanup_context(context);
                return;
            }

            loop {
                let result = GetMessageW(&mut message, ptr::null_mut(), 0, 0);
                if result <= 0 {
                    break;
                }
                let _ = TranslateMessage(&message);
                let _ = DispatchMessageW(&message);
            }

            cleanup_context(context);
        }
    }

    unsafe fn cleanup_context(context: *mut CallbackContext) {
        if let Some(hook) = (*context)
            .hook
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        {
            if UnhookWindowsHookEx(hook) == 0 {
                tracing::warn!(
                    "Failed to unhook Windows native hotkey keyboard hook: {}",
                    GetLastError()
                );
            }
        }
        let _ = HOOK_CONTEXT.compare_exchange(
            context,
            ptr::null_mut(),
            AtomicOrdering::SeqCst,
            AtomicOrdering::SeqCst,
        );
        drop(Box::from_raw(context));
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code == HC_ACTION as i32 && lparam != 0 {
            let keyboard = *(lparam as *const KBDLLHOOKSTRUCT);
            let injected = (keyboard.flags & LLKHF_INJECTED) != 0;
            let extended = (keyboard.flags & LLKHF_EXTENDED) != 0;
            let vk_code = normalize_vk(keyboard.vkCode, keyboard.scanCode, extended);
            if (!injected || accept_synthetic_events()) && dispatch_keyboard_event(vk_code, wparam)
            {
                return 1;
            }
        }

        CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
    }

    /// Korean keyboard types such as `kbd101a` (type 8, subtype 3) turn the
    /// right Alt and right Ctrl keys into 한/영 (`VK_HANGUL`) and 한자
    /// (`VK_HANJA`). Map them back by key position (extended scan code) so
    /// RightAlt / RightCtrl bindings still fire. Dedicated 한/영·한자 keys use
    /// other scan codes and are left alone.
    fn normalize_vk(vk_code: u32, scan_code: u32, extended: bool) -> u32 {
        match (vk_code, scan_code, extended) {
            (VK_HANGUL, SCAN_ALT, true) => VK_RMENU,
            (VK_HANJA, SCAN_CONTROL, true) => VK_RCONTROL,
            _ => vk_code,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn korean_layout_right_keys_map_to_right_modifiers() {
            assert_eq!(normalize_vk(VK_HANJA, SCAN_CONTROL, true), VK_RCONTROL);
            assert_eq!(normalize_vk(VK_HANGUL, SCAN_ALT, true), VK_RMENU);
        }

        #[test]
        fn other_keys_are_unchanged() {
            // Real right modifiers on a plain layout.
            assert_eq!(normalize_vk(VK_RCONTROL, SCAN_CONTROL, true), VK_RCONTROL);
            assert_eq!(normalize_vk(VK_RMENU, SCAN_ALT, true), VK_RMENU);
            // Dedicated 한/영 (0xF2) and 한자 (0xF1) keys.
            assert_eq!(normalize_vk(VK_HANGUL, 0xF2, false), VK_HANGUL);
            assert_eq!(normalize_vk(VK_HANJA, 0xF1, false), VK_HANJA);
            // Left Ctrl position is not extended.
            assert_eq!(normalize_vk(VK_HANJA, SCAN_CONTROL, false), VK_HANJA);
        }
    }

    fn dispatch_keyboard_event(vk_code: u32, message: WPARAM) -> bool {
        let pressed = match message {
            WM_KEYDOWN | WM_SYSKEYDOWN => true,
            WM_KEYUP | WM_SYSKEYUP => false,
            _ => return false,
        };

        let context = unsafe { callback_context() };
        let Some(context) = context else {
            return false;
        };

        let base = match vk_code {
            VK_RMENU => Some(NativeHotkeyTrigger::RightAlt),
            VK_RWIN => Some(NativeHotkeyTrigger::RightCommand),
            VK_RCONTROL => Some(NativeHotkeyTrigger::RightCtrl),
            VK_RSHIFT => Some(NativeHotkeyTrigger::RightShift),
            _ => None,
        };
        if let Some(base) = base {
            let Some(state) = context.state_for(base) else {
                return false;
            };
            let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
            return dispatch_native_base_edge(
                &mut state,
                &context.bindings,
                base,
                pressed,
                &context.handler,
            );
        }

        let combo = match vk_code {
            VK_SPACE => NativeComboKey::Space,
            VK_LSHIFT => NativeComboKey::LeftShift,
            other => match NativeKey::from_vk(other) {
                Some(key) => NativeComboKey::Key(key),
                None => return false,
            },
        };
        let mut matched = false;
        for (base, state) in &context.states {
            let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
            matched |= dispatch_native_combo_edge(
                &mut state,
                &context.bindings,
                *base,
                combo,
                pressed,
                &context.handler,
            );
        }
        matched
    }

    unsafe fn callback_context<'a>() -> Option<&'a CallbackContext> {
        let ptr = HOOK_CONTEXT.load(AtomicOrdering::SeqCst);
        if ptr.is_null() {
            None
        } else {
            Some(&*ptr)
        }
    }

    fn accept_synthetic_events() -> bool {
        std::env::var(ACCEPT_SYNTHETIC_EVENTS_ENV).ok().as_deref() == Some("1")
    }
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
mod platform {
    use super::{NativeHotkeyBinding, NativeHotkeyHandler};

    pub struct PlatformNativeMonitor;

    impl PlatformNativeMonitor {
        pub fn start(
            _bindings: Vec<NativeHotkeyBinding>,
            _hold_dictation: bool,
            _handler: NativeHotkeyHandler,
        ) -> Result<Self, String> {
            Err("Native hotkey runtime is unsupported on this platform".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_state_dedupes_repeat_edges() {
        let state = NativeHeldState::default();

        assert_eq!(state.edge(true), Some(ShortcutState::Pressed));
        assert_eq!(state.edge(true), None);
        assert_eq!(state.edge(false), Some(ShortcutState::Released));
        assert_eq!(state.edge(false), None);
        assert_eq!(state.edge(true), Some(ShortcutState::Pressed));
    }

    #[test]
    fn runtime_install_accepts_shared_handler_arc() {
        let runtime = NativeHotkeyRuntime::default();
        let handler: Arc<dyn Fn(NativeHotkeyEvent) + Send + Sync> = Arc::new(|_| {});

        assert!(runtime.install(Vec::new(), false, handler).is_ok());
    }

    type RecordedEvents = Arc<Mutex<Vec<(crate::hotkey::HotkeyRole, ShortcutState)>>>;

    fn right_command_bindings() -> (
        Vec<NativeMonitoredBinding>,
        RecordedEvents,
        NativeHotkeyHandler,
    ) {
        let bindings = monitored_bindings_for_bases(
            vec![NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Dictation,
                index: 0,
                trigger: NativeHotkeyTrigger::RightCommand,
            }],
            &[NativeHotkeyTrigger::RightCommand],
        );
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let handler: NativeHotkeyHandler = Arc::new(move |event| {
            captured.lock().unwrap().push((event.role, event.state));
        });
        (bindings, events, handler)
    }

    #[test]
    fn deferred_modifier_fires_on_lone_tap() {
        let (bindings, events, handler) = right_command_bindings();
        let base = NativeHotkeyTrigger::RightCommand;
        let mut state = NativeComboState {
            defer_bare: true,
            ..NativeComboState::default()
        };

        dispatch_native_base_edge(&mut state, &bindings, base, true, &handler);
        assert!(events.lock().unwrap().is_empty(), "press alone must wait");
        dispatch_native_base_edge(&mut state, &bindings, base, false, &handler);

        assert_eq!(
            *events.lock().unwrap(),
            vec![
                (crate::hotkey::HotkeyRole::Dictation, ShortcutState::Pressed),
                (
                    crate::hotkey::HotkeyRole::Dictation,
                    ShortcutState::Released
                ),
            ]
        );
    }

    #[test]
    fn deferred_modifier_used_in_shortcut_does_not_fire() {
        let (bindings, events, handler) = right_command_bindings();
        let base = NativeHotkeyTrigger::RightCommand;
        let mut state = NativeComboState {
            defer_bare: true,
            ..NativeComboState::default()
        };

        dispatch_native_base_edge(&mut state, &bindings, base, true, &handler);
        cancel_native_base_on_other_key(&mut state, &bindings, base, &handler);
        dispatch_native_base_edge(&mut state, &bindings, base, false, &handler);
        assert!(events.lock().unwrap().is_empty());

        // The next lone tap works again.
        dispatch_native_base_edge(&mut state, &bindings, base, true, &handler);
        dispatch_native_base_edge(&mut state, &bindings, base, false, &handler);
        assert_eq!(events.lock().unwrap().len(), 2);
    }

    #[test]
    fn deferred_modifier_hold_stops_when_another_key_is_pressed() {
        let (bindings, events, handler) = right_command_bindings();
        let base = NativeHotkeyTrigger::RightCommand;
        let mut state = NativeComboState {
            defer_bare: true,
            hold_base: true,
            ..NativeComboState::default()
        };

        dispatch_native_base_edge(&mut state, &bindings, base, true, &handler);
        let generation = state.edge_generation;
        assert!(activate_pending_hold_base(
            &mut state, &bindings, base, generation, &handler
        ));
        cancel_native_base_on_other_key(&mut state, &bindings, base, &handler);
        dispatch_native_base_edge(&mut state, &bindings, base, false, &handler);

        assert_eq!(
            *events.lock().unwrap(),
            vec![
                (crate::hotkey::HotkeyRole::Dictation, ShortcutState::Pressed),
                (
                    crate::hotkey::HotkeyRole::Dictation,
                    ShortcutState::Released
                ),
            ]
        );
    }

    #[test]
    fn right_command_combo_fires_without_the_bare_binding() {
        let slash = NativeHotkeyTrigger::RightCombo(
            RightModifier::Command,
            NativeComboKey::Key(NativeKey::from_name("/").unwrap()),
        );
        let bindings = monitored_bindings_for_bases(
            vec![
                NativeHotkeyBinding {
                    role: crate::hotkey::HotkeyRole::Dictation,
                    index: 0,
                    trigger: NativeHotkeyTrigger::RightCommand,
                },
                NativeHotkeyBinding {
                    role: crate::hotkey::HotkeyRole::Ask,
                    index: 0,
                    trigger: slash,
                },
            ],
            &[NativeHotkeyTrigger::RightCommand],
        );
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let handler: NativeHotkeyHandler = Arc::new(move |event| {
            captured.lock().unwrap().push((event.role, event.state));
        });
        let base = NativeHotkeyTrigger::RightCommand;
        let combo = NativeComboKey::Key(NativeKey::from_name("/").unwrap());
        let other = NativeComboKey::Key(NativeKey::from_name("c").unwrap());
        let mut state = NativeComboState {
            defer_bare: true,
            ..NativeComboState::default()
        };

        dispatch_native_base_edge(&mut state, &bindings, base, true, &handler);
        assert!(!dispatch_native_combo_edge(
            &mut state, &bindings, base, other, true, &handler
        ));
        assert!(dispatch_native_combo_edge(
            &mut state, &bindings, base, combo, true, &handler
        ));
        assert!(dispatch_native_combo_edge(
            &mut state, &bindings, base, combo, false, &handler
        ));
        dispatch_native_base_edge(&mut state, &bindings, base, false, &handler);

        assert_eq!(
            *events.lock().unwrap(),
            vec![
                (crate::hotkey::HotkeyRole::Ask, ShortcutState::Pressed),
                (crate::hotkey::HotkeyRole::Ask, ShortcutState::Released),
            ]
        );
    }

    #[test]
    fn native_key_names_round_trip() {
        for name in ["A", "z", "0", "/", ",", "\\", "Enter", "F12", "Up"] {
            let key = NativeKey::from_name(name).unwrap();
            assert!(key.name().eq_ignore_ascii_case(name));
        }
        assert_eq!(NativeKey::from_name("Space"), None);
        assert_eq!(
            NativeComboKey::from_name("space"),
            Some(NativeComboKey::Space)
        );
        assert_eq!(
            NativeHotkeyTrigger::from_base_combo(
                NativeHotkeyTrigger::RightShift,
                NativeComboKey::LeftShift
            ),
            None
        );
    }

    #[test]
    fn bound_bases_lists_each_base_once() {
        let bindings = monitored_bindings_for_bases(
            vec![
                NativeHotkeyBinding {
                    role: crate::hotkey::HotkeyRole::Dictation,
                    index: 0,
                    trigger: NativeHotkeyTrigger::Fn,
                },
                NativeHotkeyBinding {
                    role: crate::hotkey::HotkeyRole::Ask,
                    index: 0,
                    trigger: NativeHotkeyTrigger::FnSpace,
                },
                NativeHotkeyBinding {
                    role: crate::hotkey::HotkeyRole::TranslateSelection,
                    index: 0,
                    trigger: NativeHotkeyTrigger::RightShift,
                },
                NativeHotkeyBinding {
                    role: crate::hotkey::HotkeyRole::TranslateSelection,
                    index: 1,
                    trigger: NativeHotkeyTrigger::RightAltSpace,
                },
            ],
            &[NativeHotkeyTrigger::Fn, NativeHotkeyTrigger::RightShift],
        );
        assert_eq!(bindings.len(), 3);
        assert_eq!(
            bound_bases(&bindings),
            vec![NativeHotkeyTrigger::Fn, NativeHotkeyTrigger::RightShift]
        );
    }

    #[test]
    fn combo_state_suppresses_bare_base_when_combo_is_used() {
        let bindings = vec![
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Dictation,
                index: 0,
                trigger: NativeHotkeyTrigger::Fn,
            }),
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Ask,
                index: 0,
                trigger: NativeHotkeyTrigger::FnSpace,
            }),
        ];
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let handler: NativeHotkeyHandler = Arc::new(move |event| {
            captured.lock().unwrap().push((event.role, event.state));
        });
        let mut state = NativeComboState::default();

        assert!(dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            true,
            &handler,
        ));
        assert!(events.lock().unwrap().is_empty());

        assert!(dispatch_native_combo_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            NativeComboKey::Space,
            true,
            &handler,
        ));
        assert!(dispatch_native_combo_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            NativeComboKey::Space,
            false,
            &handler,
        ));
        assert!(dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            false,
            &handler,
        ));

        assert_eq!(
            *events.lock().unwrap(),
            vec![
                (crate::hotkey::HotkeyRole::Ask, ShortcutState::Pressed),
                (crate::hotkey::HotkeyRole::Ask, ShortcutState::Released),
            ]
        );
    }

    #[test]
    fn combo_state_dispatches_bare_base_when_no_combo_is_used() {
        let bindings = vec![
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Dictation,
                index: 0,
                trigger: NativeHotkeyTrigger::Fn,
            }),
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Ask,
                index: 0,
                trigger: NativeHotkeyTrigger::FnSpace,
            }),
        ];
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let handler: NativeHotkeyHandler = Arc::new(move |event| {
            captured.lock().unwrap().push((event.role, event.state));
        });
        let mut state = NativeComboState::default();

        assert!(dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            true,
            &handler,
        ));
        assert!(events.lock().unwrap().is_empty());

        assert!(dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            false,
            &handler,
        ));

        assert_eq!(
            *events.lock().unwrap(),
            vec![
                (crate::hotkey::HotkeyRole::Dictation, ShortcutState::Pressed),
                (
                    crate::hotkey::HotkeyRole::Dictation,
                    ShortcutState::Released
                ),
            ]
        );
    }

    #[test]
    fn hold_fn_starts_while_held_and_stops_on_release() {
        let bindings = vec![
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Dictation,
                index: 0,
                trigger: NativeHotkeyTrigger::Fn,
            }),
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Ask,
                index: 0,
                trigger: NativeHotkeyTrigger::FnSpace,
            }),
        ];
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let handler: NativeHotkeyHandler = Arc::new(move |event| {
            captured.lock().unwrap().push((event.role, event.state));
        });
        let mut state = NativeComboState {
            hold_base: true,
            ..NativeComboState::default()
        };

        assert!(dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            true,
            &handler,
        ));
        assert!(events.lock().unwrap().is_empty());

        let generation = state.edge_generation;
        assert!(activate_pending_hold_base(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            generation,
            &handler,
        ));
        assert_eq!(
            *events.lock().unwrap(),
            vec![(crate::hotkey::HotkeyRole::Dictation, ShortcutState::Pressed)]
        );

        assert!(dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            false,
            &handler,
        ));
        assert_eq!(
            *events.lock().unwrap(),
            vec![
                (crate::hotkey::HotkeyRole::Dictation, ShortcutState::Pressed),
                (
                    crate::hotkey::HotkeyRole::Dictation,
                    ShortcutState::Released
                ),
            ]
        );
    }

    #[test]
    fn fn_combo_cancels_pending_hold_and_short_tap_does_not_start() {
        let bindings = vec![
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Dictation,
                index: 0,
                trigger: NativeHotkeyTrigger::Fn,
            }),
            NativeMonitoredBinding::new(NativeHotkeyBinding {
                role: crate::hotkey::HotkeyRole::Ask,
                index: 0,
                trigger: NativeHotkeyTrigger::FnSpace,
            }),
        ];
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let handler: NativeHotkeyHandler = Arc::new(move |event| {
            captured.lock().unwrap().push((event.role, event.state));
        });
        let mut state = NativeComboState {
            hold_base: true,
            ..NativeComboState::default()
        };

        let _ = dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            true,
            &handler,
        );
        let stale_generation = state.edge_generation;
        let _ = dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            false,
            &handler,
        );
        assert!(!activate_pending_hold_base(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            stale_generation,
            &handler,
        ));
        assert!(events.lock().unwrap().is_empty());

        let _ = dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            true,
            &handler,
        );
        let generation = state.edge_generation;
        let _ = dispatch_native_combo_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            NativeComboKey::Space,
            true,
            &handler,
        );
        assert!(!activate_pending_hold_base(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            generation,
            &handler,
        ));
        let _ = dispatch_native_base_edge(
            &mut state,
            &bindings,
            NativeHotkeyTrigger::Fn,
            false,
            &handler,
        );
        assert_eq!(
            *events.lock().unwrap(),
            vec![
                (crate::hotkey::HotkeyRole::Ask, ShortcutState::Pressed),
                (crate::hotkey::HotkeyRole::Ask, ShortcutState::Released),
            ]
        );
    }
}
