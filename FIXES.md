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
- `com.runningwithcrayons.Alfred` và `com.runningwithcrayons.Alfred-3`
- `com.raycast.macos`

Thứ tự:

1. Accessibility API sửa đúng đoạn đang soạn trong ô đang focus (timeout 80ms).
2. Nếu Accessibility không trả lời, Shift+Left bôi đen số ký tự cần thay, rồi gõ đè.

Các app khác vẫn dùng Backspace như bản gốc.

## Build trên Mac

1. Mở `Sources/OpenKey/macOS/OpenKey.xcodeproj`.
2. Product → Archive → Distribute App.
3. Cấp quyền Accessibility cho app mới, tắt bộ gõ tiếng Việt khác.

Máy Linux không build được app native macOS này.
