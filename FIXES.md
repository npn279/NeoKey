# OpenKey 2.0.6

Fork của [tuyenvm/OpenKey](https://github.com/tuyenvm/OpenKey) (GPL-3.0). Hai lỗi trên macOS:

## Lag, bật tiếng Việt rồi không gõ được

Mỗi phím trước đây gọi `NSWorkspace frontmostApplication` và `CGWindowListCopyWindowInfo`. Hai lệnh này là IPC nặng. Callback của event tap chạy quá lâu thì macOS tắt tap (`kCGEventTapDisabledByTimeout`) và bộ gõ im cho đến khi mở lại app.

- Bundle id của app nhận phím được lấy từ `kCGEventTargetUnixProcessID` và cache theo PID. `NSWorkspace` chỉ chạy khi PID đổi hoặc khi đổi app.
- Khi tap bị tắt vì timeout hoặc user input, callback bật lại ngay.
- Watchdog mỗi giây gọi `CGEventTapIsEnabled` và bật lại nếu tap đang tắt.

## Spotlight gõ `ồ` thành `oồ`

Ô tìm kiếm của Spotlight, Alfred và Raycast nuốt Backspace mô phỏng, nên nguyên âm cũ còn lại và chữ có dấu được chèn thêm (`o` + `ồ`).

Với các bundle id sau, OpenKey không gửi Backspace:

- `com.apple.Spotlight`
- `com.apple.campo` (Siri AI trên macOS 27)
- `com.runningwithcrayons.Alfred` và `com.runningwithcrayons.Alfred-3`
- `com.raycast.macos`

Trên macOS 27, Spotlight giữ nguyên ứng dụng tiền cảnh nên PID của sự kiện phím có thể thuộc ứng dụng bên dưới. Cả bản Rust và Objective-C chỉ hỏi Accessibility về chủ sở hữu ô đang focus khi bộ gõ cần xóa ký tự. Nhờ đó Telex gõ `oo` cho ra `ô`. Kiểm tra trực tiếp bằng `rust/neokey-mac/live-tests/spotlight_oo.sh`.

Thứ tự:

1. Accessibility API sửa đúng đoạn đang soạn trong ô đang focus (timeout 80ms).
2. Nếu Accessibility không trả lời, Shift+Left bôi đen số ký tự cần thay, rồi gõ đè.

Các app khác vẫn dùng Backspace như bản gốc.

## Build trên Mac

App Objective-C đã được thay bằng bản Rust trong `rust/neokey-mac`, giữ nguyên hai cách xử lý trên. Xem [rust/README.md](rust/README.md):

1. Trong `rust/`, chạy `./neokey-mac/bundle.sh`.
2. Mở `target/NeoKey.app`, cấp quyền Accessibility, tắt bộ gõ tiếng Việt khác.
