use objc2::runtime::AnyClass;
use objc2::sel;
use objc2_service_management::{SMAppService, SMAppServiceStatus};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LoginStatus {
    Unavailable,
    Off,
    Enabled,
    NeedsApproval,
}

fn service(bundled: bool) -> Option<objc2::rc::Retained<SMAppService>> {
    if !bundled
        || !AnyClass::get(c"SMAppService")
            .is_some_and(|class| class.class_method(sel!(mainAppService)).is_some())
    {
        return None;
    }
    // This selector is available starting in macOS 13.
    Some(unsafe { SMAppService::mainAppService() })
}

pub fn status(bundled: bool) -> LoginStatus {
    let Some(service) = service(bundled) else {
        return LoginStatus::Unavailable;
    };
    match unsafe { service.status() } {
        SMAppServiceStatus::Enabled => LoginStatus::Enabled,
        SMAppServiceStatus::RequiresApproval => LoginStatus::NeedsApproval,
        SMAppServiceStatus::NotFound => LoginStatus::Unavailable,
        _ => LoginStatus::Off,
    }
}

pub fn set_enabled(bundled: bool, enabled: bool) -> Result<(), String> {
    let Some(service) = service(bundled) else {
        return Ok(());
    };
    let status = unsafe { service.status() };
    let result = if enabled
        && status != SMAppServiceStatus::Enabled
        && status != SMAppServiceStatus::RequiresApproval
    {
        unsafe { service.registerAndReturnError() }
    } else if !enabled
        && matches!(
            status,
            SMAppServiceStatus::Enabled | SMAppServiceStatus::RequiresApproval
        )
    {
        unsafe { service.unregisterAndReturnError() }
    } else {
        Ok(())
    };
    result.map_err(|error| error.localizedDescription().to_string())
}

pub fn open_settings() {
    if AnyClass::get(c"SMAppService").is_some_and(|class| {
        class
            .class_method(sel!(openSystemSettingsLoginItems))
            .is_some()
    }) {
        unsafe { SMAppService::openSystemSettingsLoginItems() };
    }
}
