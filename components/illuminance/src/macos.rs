//! macOS ambient light sensor support.
//!
//! Uses the private `IOHIDEventSystemClient` API to poll the ambient light
//! sensor HID service. This works on both Apple Silicon and modern Intel Macs.
//! The service is matched by its HID usage (`PrimaryUsagePage = 0xff00`,
//! `PrimaryUsage = 4`) and queried for an `kIOHIDEventTypeAmbientLightSensor`
//! event, whose level field carries the lux value. Because these symbols are
//! SPI (not present in the public SDK stub), they are resolved at runtime via
//! `dlsym` rather than linked.

use std::os::raw::{c_int, c_void};

pub(crate) async fn read_illuminance(_device_path: Option<&String>) -> Result<f64, String> {
    // The underlying IOKit/HID calls are synchronous; run them on a blocking
    // thread so we don't stall the async runtime.
    tokio::task::spawn_blocking(read_via_hid)
        .await
        .map_err(|e| format!("Light sensor task failed: {}", e))?
}

// kIOHIDEventTypeAmbientLightSensor
const K_IOHID_EVENT_TYPE_ALS: i64 = 12;
// IOHIDEventFieldBase(type) == type << 16; this addresses the ALS "level" (lux) field.
const K_IOHID_FIELD_ALS_LEVEL: i32 = (K_IOHID_EVENT_TYPE_ALS as i32) << 16;

fn read_via_hid() -> Result<f64, String> {
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
    use core_foundation_sys::base::{kCFAllocatorDefault, CFAllocatorRef, CFRelease, CFTypeRef};
    use core_foundation_sys::dictionary::CFDictionaryRef;

    // SPI function signatures, resolved at runtime via dlsym.
    type ClientCreateFn = unsafe extern "C" fn(CFAllocatorRef) -> *mut c_void;
    type ClientSetMatchingFn = unsafe extern "C" fn(*mut c_void, CFDictionaryRef) -> c_int;
    type ClientCopyServicesFn = unsafe extern "C" fn(*mut c_void) -> CFArrayRef;
    type ServiceCopyEventFn = unsafe extern "C" fn(*const c_void, i64, i32, i64) -> *mut c_void;
    type EventGetFloatValueFn = unsafe extern "C" fn(*mut c_void, i32) -> f64;

    // Releases a CoreFoundation object on drop, so early returns don't leak.
    struct ReleaseGuard(CFTypeRef);
    impl Drop for ReleaseGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CFRelease(self.0) };
            }
        }
    }

    unsafe fn dlopen_iokit() -> Result<*mut c_void, String> {
        let handle = libc::dlopen(
            c"/System/Library/Frameworks/IOKit.framework/IOKit".as_ptr(),
            libc::RTLD_NOW,
        );
        if handle.is_null() {
            return Err("Failed to dlopen IOKit framework".to_string());
        }
        Ok(handle)
    }

    unsafe fn load_sym(handle: *mut c_void, name: &std::ffi::CStr) -> Result<*mut c_void, String> {
        let sym = libc::dlsym(handle, name.as_ptr());
        if sym.is_null() {
            return Err(format!(
                "Symbol {} not available in IOKit",
                name.to_string_lossy()
            ));
        }
        Ok(sym)
    }

    unsafe {
        let iokit = dlopen_iokit()?;

        let client_create: ClientCreateFn =
            std::mem::transmute(load_sym(iokit, c"IOHIDEventSystemClientCreate")?);
        let client_set_matching: ClientSetMatchingFn =
            std::mem::transmute(load_sym(iokit, c"IOHIDEventSystemClientSetMatching")?);
        let client_copy_services: ClientCopyServicesFn =
            std::mem::transmute(load_sym(iokit, c"IOHIDEventSystemClientCopyServices")?);
        let service_copy_event: ServiceCopyEventFn =
            std::mem::transmute(load_sym(iokit, c"IOHIDServiceClientCopyEvent")?);
        let event_get_float_value: EventGetFloatValueFn =
            std::mem::transmute(load_sym(iokit, c"IOHIDEventGetFloatValue")?);

        let client = client_create(kCFAllocatorDefault);
        if client.is_null() {
            return Err("Failed to create IOHIDEventSystemClient".to_string());
        }
        // RAII-ish cleanup for the client.
        let _client_guard = ReleaseGuard(client as CFTypeRef);

        // Match the ambient light sensor HID service.
        let matching = CFDictionary::from_CFType_pairs(&[
            (
                CFString::from_static_string("PrimaryUsagePage").as_CFType(),
                CFNumber::from(0xff00_i32).as_CFType(),
            ),
            (
                CFString::from_static_string("PrimaryUsage").as_CFType(),
                CFNumber::from(4_i32).as_CFType(),
            ),
        ]);
        client_set_matching(client, matching.as_concrete_TypeRef());

        let services = client_copy_services(client);
        if services.is_null() {
            return Err("IOHIDEventSystemClientCopyServices returned null".to_string());
        }
        let _services_guard = ReleaseGuard(services as CFTypeRef);

        let count = CFArrayGetCount(services);
        if count == 0 {
            return Err("No ambient light sensor HID service found".to_string());
        }

        // Borrowed reference owned by the array — do not release.
        let service = CFArrayGetValueAtIndex(services, 0) as *const c_void;
        if service.is_null() {
            return Err("Ambient light sensor service was null".to_string());
        }

        let event = service_copy_event(service, K_IOHID_EVENT_TYPE_ALS, 0, 0);
        if event.is_null() {
            return Err("Ambient light sensor returned no event".to_string());
        }
        let _event_guard = ReleaseGuard(event as CFTypeRef);

        let lux = event_get_float_value(event, K_IOHID_FIELD_ALS_LEVEL);
        Ok(lux.max(0.0))
    }
}
