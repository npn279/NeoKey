## Cách build NeoKey cho macOS

NeoKey cho macOS được viết bằng Rust (`rust/neokey-mac`). Không cần Xcode project.

Yêu cầu:
- macOS 13 trở lên (khởi động cùng máy dùng `SMAppService`).
- Rust (cài bằng `rustup`) và Command Line Tools của Xcode (`xcode-select --install`).

Build và mở app:

```sh
cd rust
cargo test --workspace
./neokey-mac/bundle.sh
open target/NeoKey.app
```

`bundle.sh` build bản release và ký ad-hoc `target/NeoKey.app`. Lần chạy đầu, NeoKey mở cửa sổ hướng dẫn cấp quyền
**System Settings → Privacy & Security → Accessibility**. Hãy tắt các bộ gõ tiếng Việt khác khi dùng NeoKey.

Bản Windows (`rust/neokey-win`) build trên Windows bằng `cargo build --release -p neokey-win`. Xem [rust/README.md](rust/README.md).
