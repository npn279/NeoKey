use objc2::rc::Retained;
use objc2::{MainThreadMarker, Message};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSWindow};
use objc2_foundation::ns_string;

use super::ui::{alert, CaseMode, ConvertUiOptions};
use super::{clipboard, login_item, updates, with_state, LoginStatus, Preferences, State, Update};

pub(super) fn load_convert_options(preferences: &Preferences) -> ConvertUiOptions {
    let case_mode = [
        (CaseMode::Upper, ns_string!("convertToolToAllCaps")),
        (CaseMode::Lower, ns_string!("convertToolToAllNonCaps")),
        (
            CaseMode::Sentence,
            ns_string!("convertToolToCapsFirstLetter"),
        ),
        (CaseMode::Title, ns_string!("convertToolToCapsEachWord")),
    ]
    .into_iter()
    .find(|(_, key)| preferences.bool(key))
    .map_or(CaseMode::Keep, |(mode, _)| mode);
    ConvertUiOptions {
        from_code: preferences
            .get(ns_string!("convertToolFromCode"))
            .clamp(0, 4) as u8,
        to_code: preferences.get(ns_string!("convertToolToCode")).clamp(0, 4) as u8,
        case_mode,
        remove_mark: preferences.bool(ns_string!("convertToolRemoveMark")),
    }
}

impl State {
    pub(super) fn save_convert_options(&self) {
        let Some(ui) = &self.ui else { return };
        let options = ui.settings.convert.ui_options();
        for (key, value) in [
            (
                ns_string!("convertToolToAllCaps"),
                i64::from(options.case_mode == CaseMode::Upper),
            ),
            (
                ns_string!("convertToolToAllNonCaps"),
                i64::from(options.case_mode == CaseMode::Lower),
            ),
            (
                ns_string!("convertToolToCapsFirstLetter"),
                i64::from(options.case_mode == CaseMode::Sentence),
            ),
            (
                ns_string!("convertToolToCapsEachWord"),
                i64::from(options.case_mode == CaseMode::Title),
            ),
            (
                ns_string!("convertToolRemoveMark"),
                i64::from(options.remove_mark),
            ),
            (
                ns_string!("convertToolFromCode"),
                i64::from(options.from_code),
            ),
            (ns_string!("convertToolToCode"), i64::from(options.to_code)),
        ] {
            self.preferences.integer(key, value);
        }
    }

    pub(super) fn schedule_quick_convert(&mut self) {
        if std::mem::replace(&mut self.quick_convert_pending, true) {
            return;
        }
        alert::defer(|| {
            let Some((options, notify, parent)) = with_state(|state| {
                state.quick_convert_pending = false;
                if let Some(ui) = &state.ui {
                    ui.panel.close();
                }
                (
                    load_convert_options(&state.preferences),
                    state
                        .preferences
                        .bool(ns_string!("convertToolDontAlertWhenCompleted")),
                    state.settings_window(),
                )
            }) else {
                return;
            };
            let (message, detail) = match clipboard::quick_convert(options) {
                clipboard::Result::Converted if notify => (
                    "Chuyển mã thành công!",
                    "Kết quả đã được lưu trong clipboard.",
                ),
                clipboard::Result::Converted => return,
                clipboard::Result::Empty => (
                    "Không có dữ liệu trong clipboard!",
                    "Hãy sao chép một đoạn văn bản để chuyển đổi.",
                ),
                clipboard::Result::Failed => (
                    "Không thể chuyển mã",
                    "Hãy kiểm tra bảng mã nguồn và thử lại.",
                ),
            };
            alert::ask(parent, message.into(), detail, "OK", None);
        });
    }

    pub(super) fn refresh_login(&mut self) {
        let status = login_item::status(self.bundle_path.is_some());
        if self.login_status != status {
            self.login_status = status;
            if let Some(ui) = &self.ui {
                ui.settings.general.refresh_login(status);
            }
        }
    }

    pub(super) fn set_login(&mut self, enabled: bool) {
        if let Err(error) = login_item::set_enabled(self.bundle_path.is_some(), enabled) {
            alert::ask(
                self.settings_window(),
                format!("Không thể thay đổi mục đăng nhập\n{error}"),
                "",
                "OK",
                None,
            );
        }
        self.login_status = login_item::status(self.bundle_path.is_some());
        if matches!(self.login_status, LoginStatus::Enabled | LoginStatus::Off) {
            self.preferences.integer(
                ns_string!("RunOnStartup"),
                i64::from(self.login_status == LoginStatus::Enabled),
            );
        }
        if let Some(ui) = &self.ui {
            ui.settings.general.refresh_login(self.login_status);
        }
    }

    pub(super) fn check_updates_on_startup(&mut self) {
        if !self.preferences.bool(ns_string!("DontCheckUpdate")) {
            self.check_updates(false);
        }
    }

    pub(super) fn check_updates(&mut self, manual: bool) {
        if let Some((_, show_result)) = &mut self.update_check {
            *show_result |= manual;
        } else {
            self.update_check = Some((updates::check(), manual));
            self.refresh_ui();
        }
    }

    pub(super) fn poll_updates(&mut self) {
        let Some((receiver, manual)) = &self.update_check else {
            return;
        };
        let update = match receiver.try_recv() {
            Ok(update) => update,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Update::Unavailable,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
        };
        let manual = *manual;
        self.update_check = None;
        self.refresh_ui();
        let parent = self.settings_window();
        match update {
            Update::Newer(version) => alert::ask(
                parent,
                format!("NeoKey có phiên bản mới ({version})"),
                "Mở GitHub để xem bản phát hành và tải bản cập nhật.",
                "Mở bản phát hành",
                Some(|| alert::open_url(updates::RELEASES)),
            ),
            Update::Current if manual => alert::ask(
                parent,
                "Bạn đang dùng phiên bản mới nhất!".into(),
                "",
                "OK",
                None,
            ),
            Update::Unavailable if manual => alert::ask(
                parent,
                "Không thể kiểm tra cập nhật".into(),
                "Hãy kiểm tra kết nối và thử lại.",
                "OK",
                None,
            ),
            _ => {}
        }
    }

    pub(super) fn confirm_reset(&self) {
        alert::ask(
            self.settings_window(),
            "Khôi phục thiết lập mặc định?".into(),
            "Dữ liệu gõ tắt và ngôn ngữ đã lưu theo ứng dụng sẽ được giữ lại.",
            "Khôi phục",
            Some(|| {
                with_state(State::reset_typing);
            }),
        );
    }

    fn reset_typing(&mut self) {
        self.selection_dirty = false;
        self.preferences.reset_typing();
        let (preferences, settings) = Preferences::load();
        self.spell_preference = settings.vCheckSpelling;
        self.typing.engine_mut().set_settings(settings);
        self.typing.reset_after_tap_disable();
        self.typing.set_chromium_fix(preferences.chromium_fix);
        self.preferences = preferences;
        self.refresh_input_source();
        self.set_login(true);
        if let Some(mtm) = MainThreadMarker::new() {
            NSApplication::sharedApplication(mtm)
                .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }
        self.refresh_ui();
    }

    fn settings_window(&self) -> Option<Retained<NSWindow>> {
        Some(self.ui.as_ref()?.settings.window().retain())
    }
}
