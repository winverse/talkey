//! Places the capsule on the screen the user is working on.
//!
//! With several monitors the capsule should appear where the user is looking,
//! not where it happened to be placed when the app started: the screen holding
//! the focused window, or the mouse pointer when no window is focused. This is
//! done on macOS and Windows; elsewhere the frontend keeps its own placement.

use tauri::{Monitor, WebviewWindow};

/// Gap between the capsule window and the bottom of the work area, in logical
/// pixels. Keep in step with `CAPSULE_BOTTOM_MARGIN` in `src/hooks/useCapsuleResize.ts`.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
const BOTTOM_MARGIN: f64 = 80.0;

/// A rectangle in the platform's global window coordinates, y pointing down:
/// points on macOS, physical pixels of the virtual screen on Windows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    fn contains(&self, (x, y): (f64, f64)) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    fn overlap_area(&self, other: &Rect) -> f64 {
        let width = (self.x + self.width).min(other.x + other.width) - self.x.max(other.x);
        let height = (self.y + self.height).min(other.y + other.height) - self.y.max(other.y);
        if width > 0.0 && height > 0.0 {
            width * height
        } else {
            0.0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Screen {
    pub bounds: Rect,
    pub work_area: Rect,
}

/// The monitor's bounds and work area in Tauri's physical pixels divided by `divisor`.
fn screen_in(monitor: &Monitor, divisor: f64) -> Screen {
    let position = monitor.position();
    let size = monitor.size();
    let work_area = monitor.work_area();
    Screen {
        bounds: Rect {
            x: f64::from(position.x) / divisor,
            y: f64::from(position.y) / divisor,
            width: f64::from(size.width) / divisor,
            height: f64::from(size.height) / divisor,
        },
        work_area: Rect {
            x: f64::from(work_area.position.x) / divisor,
            y: f64::from(work_area.position.y) / divisor,
            width: f64::from(work_area.size.width) / divisor,
            height: f64::from(work_area.size.height) / divisor,
        },
    }
}

/// Index of the screen showing most of the focused window, else of the one under the pointer.
pub(crate) fn active_screen(
    screens: &[Screen],
    focused_window: Option<Rect>,
    pointer: Option<(f64, f64)>,
) -> Option<usize> {
    let by_window = focused_window.and_then(|window| {
        screens
            .iter()
            .enumerate()
            .map(|(index, screen)| (index, screen.bounds.overlap_area(&window)))
            .filter(|(_, area)| *area > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    });
    by_window.or_else(|| {
        let pointer = pointer?;
        screens
            .iter()
            .position(|screen| screen.bounds.contains(pointer))
    })
}

/// Top-left corner that puts a window of this size at the bottom centre of the
/// work area, `margin` above its bottom edge, kept inside the work area.
#[cfg_attr(
    not(any(target_os = "macos", target_os = "windows", test)),
    allow(dead_code)
)]
pub(crate) fn bottom_center(work_area: Rect, width: f64, height: f64, margin: f64) -> (f64, f64) {
    let x = work_area.x + (work_area.width - width) / 2.0;
    let y = work_area.y + work_area.height - height - margin;
    let max_x = (work_area.x + work_area.width - width).max(work_area.x);
    let max_y = (work_area.y + work_area.height - height).max(work_area.y);
    (
        x.clamp(work_area.x, max_x).round(),
        y.clamp(work_area.y, max_y).round(),
    )
}

/// Resizes the capsule to `width` x `height` (logical) and moves it to the
/// bottom centre of the active screen. Returns false, leaving the window
/// untouched, when the active screen cannot be told.
#[tauri::command]
pub fn place_capsule_on_active_screen(window: WebviewWindow, width: f64, height: f64) -> bool {
    if window.label() != "capsule" {
        return false;
    }
    let Ok(monitors) = window.available_monitors() else {
        return false;
    };
    let screens: Vec<Screen> = monitors.iter().map(platform::screen).collect();
    let Some(index) = active_screen(
        &screens,
        platform::focused_window_frame(),
        platform::pointer_location(),
    ) else {
        return false;
    };
    platform::place(
        &window,
        &screens[index],
        monitors[index].scale_factor(),
        width,
        height,
    )
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::c_void;

    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};
    use tauri::{LogicalPosition, LogicalSize, Monitor, WebviewWindow};

    use super::{bottom_center, screen_in, Rect, Screen, BOTTOM_MARGIN};

    /// Tauri reports each macOS monitor in physical pixels scaled by its own
    /// factor, so pixels from different monitors cannot be compared. Windows
    /// are placed in points, which all screens share.
    pub(super) fn screen(monitor: &Monitor) -> Screen {
        screen_in(monitor, monitor.scale_factor())
    }

    pub(super) fn place(
        window: &WebviewWindow,
        screen: &Screen,
        _scale_factor: f64,
        width: f64,
        height: f64,
    ) -> bool {
        // Size first: a resize keeps the bottom-left corner, so it would undo
        // a position set before it.
        if window.set_size(LogicalSize::new(width, height)).is_err() {
            return false;
        }
        let (x, y) = bottom_center(screen.work_area, width, height, BOTTOM_MARGIN);
        window.set_position(LogicalPosition::new(x, y)).is_ok()
    }

    type CFTypeRef = *const c_void;

    #[repr(C)]
    #[derive(Default)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Default)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Default)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
    const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;
    const K_CG_NULL_WINDOW_ID: u32 = 0;
    const K_CF_NUMBER_SINT32_TYPE: isize = 3;
    const K_CF_NUMBER_FLOAT64_TYPE: isize = 6;
    /// Skips the tiny helper windows some apps keep on screen.
    const MIN_WINDOW_SIDE: f64 = 40.0;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFArrayGetCount(array: CFTypeRef) -> isize;
        fn CFArrayGetValueAtIndex(array: CFTypeRef, index: isize) -> CFTypeRef;
        fn CFDictionaryGetValue(dictionary: CFTypeRef, key: CFTypeRef) -> CFTypeRef;
        fn CFNumberGetValue(number: CFTypeRef, number_type: isize, value: *mut c_void) -> u8;
        fn CFRelease(value: CFTypeRef);
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        static kCGWindowOwnerPID: CFTypeRef;
        static kCGWindowLayer: CFTypeRef;
        static kCGWindowBounds: CFTypeRef;
        static kCGWindowAlpha: CFTypeRef;
        fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFTypeRef;
        fn CGRectMakeWithDictionaryRepresentation(dictionary: CFTypeRef, rect: *mut CGRect) -> u8;
        fn CGEventCreate(source: CFTypeRef) -> CFTypeRef;
        fn CGEventGetLocation(event: CFTypeRef) -> CGPoint;
    }

    fn frontmost_process_id() -> Option<i32> {
        let class = AnyClass::get("NSWorkspace")?;
        unsafe {
            let workspace: *mut AnyObject = msg_send![class, sharedWorkspace];
            if workspace.is_null() {
                return None;
            }
            let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
            if app.is_null() {
                return None;
            }
            let process_id: i32 = msg_send![app, processIdentifier];
            (process_id > 0).then_some(process_id)
        }
    }

    unsafe fn number<T: Default>(
        dictionary: CFTypeRef,
        key: CFTypeRef,
        number_type: isize,
    ) -> Option<T> {
        let value = CFDictionaryGetValue(dictionary, key);
        if value.is_null() {
            return None;
        }
        let mut out = T::default();
        (CFNumberGetValue(value, number_type, (&mut out as *mut T).cast()) != 0).then_some(out)
    }

    /// Frame of the frontmost app's top window, in global points.
    ///
    /// The window list runs front to back, and the focused window is the
    /// frontmost of its app's windows. Bounds need no screen recording
    /// permission; only window titles do.
    pub(super) fn focused_window_frame() -> Option<Rect> {
        let process_id = frontmost_process_id()?;
        unsafe {
            let windows = CGWindowListCopyWindowInfo(
                K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY | K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
                K_CG_NULL_WINDOW_ID,
            );
            if windows.is_null() {
                return None;
            }
            let mut found = None;
            for index in 0..CFArrayGetCount(windows) {
                let window = CFArrayGetValueAtIndex(windows, index);
                if number::<i32>(window, kCGWindowOwnerPID, K_CF_NUMBER_SINT32_TYPE)
                    != Some(process_id)
                    || number::<i32>(window, kCGWindowLayer, K_CF_NUMBER_SINT32_TYPE) != Some(0)
                    || number::<f64>(window, kCGWindowAlpha, K_CF_NUMBER_FLOAT64_TYPE)
                        .is_some_and(|alpha| alpha <= 0.0)
                {
                    continue;
                }
                let bounds = CFDictionaryGetValue(window, kCGWindowBounds);
                let mut rect = CGRect::default();
                if bounds.is_null()
                    || CGRectMakeWithDictionaryRepresentation(bounds, &mut rect) == 0
                {
                    continue;
                }
                if rect.size.width < MIN_WINDOW_SIDE || rect.size.height < MIN_WINDOW_SIDE {
                    continue;
                }
                found = Some(Rect {
                    x: rect.origin.x,
                    y: rect.origin.y,
                    width: rect.size.width,
                    height: rect.size.height,
                });
                break;
            }
            CFRelease(windows);
            found
        }
    }

    /// Mouse pointer in global points, top-left origin like window bounds.
    pub(super) fn pointer_location() -> Option<(f64, f64)> {
        unsafe {
            let event = CGEventCreate(std::ptr::null());
            if event.is_null() {
                return None;
            }
            let point = CGEventGetLocation(event);
            CFRelease(event);
            Some((point.x, point.y))
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use tauri::{LogicalSize, Monitor, PhysicalPosition, WebviewWindow};
    use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowRect, IsIconic,
    };

    use super::{bottom_center, screen_in, Rect, Screen, BOTTOM_MARGIN};

    /// Skips the tiny helper windows some apps bring to the foreground.
    const MIN_WINDOW_SIDE: f64 = 40.0;

    /// Windows reports monitors, windows and the pointer in physical pixels of
    /// one virtual screen (Tauri apps are per-monitor DPI aware).
    pub(super) fn screen(monitor: &Monitor) -> Screen {
        screen_in(monitor, 1.0)
    }

    pub(super) fn place(
        window: &WebviewWindow,
        screen: &Screen,
        scale_factor: f64,
        width: f64,
        height: f64,
    ) -> bool {
        let (x, y) = bottom_center(
            screen.work_area,
            width * scale_factor,
            height * scale_factor,
            BOTTOM_MARGIN * scale_factor,
        );
        let position = PhysicalPosition::new(x as i32, y as i32);
        // Moving onto a monitor with another DPI makes tao rescale the window
        // and may shift it. Move first, size in logical pixels for the new
        // DPI, then settle the position again.
        window.set_position(position).is_ok()
            && window.set_size(LogicalSize::new(width, height)).is_ok()
            && window.set_position(position).is_ok()
    }

    /// The desktop and the taskbar can hold the focus; they say nothing about
    /// which screen the user is looking at.
    unsafe fn is_shell_window(window: HWND) -> bool {
        let mut class_name = [0u16; 32];
        let length = GetClassNameW(window, class_name.as_mut_ptr(), class_name.len() as i32);
        if length <= 0 {
            return false;
        }
        matches!(
            String::from_utf16_lossy(&class_name[..length as usize]).as_str(),
            "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
        )
    }

    /// Frame of the foreground window in physical pixels.
    pub(super) fn focused_window_frame() -> Option<Rect> {
        unsafe {
            let window = GetForegroundWindow();
            if window.is_null() || IsIconic(window) != 0 || is_shell_window(window) {
                return None;
            }
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(window, &mut rect) == 0 {
                return None;
            }
            let width = f64::from(rect.right - rect.left);
            let height = f64::from(rect.bottom - rect.top);
            (width >= MIN_WINDOW_SIDE && height >= MIN_WINDOW_SIDE).then_some(Rect {
                x: f64::from(rect.left),
                y: f64::from(rect.top),
                width,
                height,
            })
        }
    }

    /// Mouse pointer in physical pixels.
    pub(super) fn pointer_location() -> Option<(f64, f64)> {
        let mut point = POINT { x: 0, y: 0 };
        (unsafe { GetCursorPos(&mut point) } != 0).then(|| (f64::from(point.x), f64::from(point.y)))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use tauri::{Monitor, WebviewWindow};

    use super::{screen_in, Rect, Screen};

    pub(super) fn screen(monitor: &Monitor) -> Screen {
        screen_in(monitor, 1.0)
    }

    pub(super) fn focused_window_frame() -> Option<Rect> {
        None
    }

    pub(super) fn pointer_location() -> Option<(f64, f64)> {
        None
    }

    pub(super) fn place(
        _window: &WebviewWindow,
        _screen: &Screen,
        _scale_factor: f64,
        _width: f64,
        _height: f64,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Built-in Retina display in the middle, 1440p on the left, 1080p on the right.
    fn three_screens() -> Vec<Screen> {
        vec![
            Screen {
                bounds: rect(-2560.0, -200.0, 2560.0, 1440.0),
                work_area: rect(-2560.0, -200.0, 2560.0, 1440.0),
            },
            Screen {
                bounds: rect(0.0, 0.0, 1728.0, 1117.0),
                work_area: rect(0.0, 38.0, 1728.0, 1009.0),
            },
            Screen {
                bounds: rect(1728.0, 0.0, 1920.0, 1080.0),
                work_area: rect(1728.0, 25.0, 1920.0, 1055.0),
            },
        ]
    }

    #[test]
    fn picks_the_screen_showing_most_of_the_focused_window() {
        let screens = three_screens();
        // Mostly on the right screen, a little over the middle one.
        let window = rect(1600.0, 100.0, 1000.0, 600.0);
        let pointer = (-100.0, 300.0);

        assert_eq!(
            active_screen(&screens, Some(window), Some(pointer)),
            Some(2)
        );
    }

    #[test]
    fn picks_a_screen_left_of_the_main_one() {
        let screens = three_screens();
        let window = rect(-2000.0, 0.0, 1200.0, 800.0);

        assert_eq!(active_screen(&screens, Some(window), None), Some(0));
    }

    #[test]
    fn falls_back_to_the_pointer_without_a_focused_window() {
        let screens = three_screens();

        assert_eq!(
            active_screen(&screens, None, Some((2000.0, 500.0))),
            Some(2)
        );
    }

    #[test]
    fn falls_back_to_the_pointer_when_the_window_is_off_screen() {
        let screens = three_screens();
        let window = rect(10_000.0, 10_000.0, 800.0, 600.0);

        assert_eq!(
            active_screen(&screens, Some(window), Some((10.0, 10.0))),
            Some(1)
        );
    }

    #[test]
    fn finds_no_screen_without_any_hint() {
        assert_eq!(active_screen(&three_screens(), None, None), None);
        assert_eq!(
            active_screen(&three_screens(), None, Some((9000.0, 0.0))),
            None
        );
    }

    #[test]
    fn centres_the_capsule_above_the_bottom_of_the_work_area() {
        let work_area = rect(1728.0, 25.0, 1920.0, 1055.0);

        assert_eq!(bottom_center(work_area, 84.0, 84.0, 80.0), (2646.0, 916.0));
    }

    #[test]
    fn keeps_the_capsule_inside_a_small_work_area() {
        let work_area = rect(0.0, 0.0, 300.0, 100.0);

        assert_eq!(bottom_center(work_area, 384.0, 84.0, 80.0), (0.0, 0.0));
    }
}
