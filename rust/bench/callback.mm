#include "OpenKey.mm"
#include <dlfcn.h>

ViewController* viewController = nil;
AppDelegate* appDelegate = nil;
int vSendKeyStepByStep = 0;
int vFixChromiumBrowser = 0;
int vPerformLayoutCompat = 0;

static constexpr pid_t targetPid = 4242;
static uint64_t posted;
static bool recording;
static std::vector<uint16_t> screen;

static void appendPhysical(CGEventRef event) {
    auto key = static_cast<Uint32>(CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode));
    auto flags = CGEventGetFlags(event);
    if (flags & (kCGEventFlagMaskCommand | kCGEventFlagMaskControl)) return;
    if (key == KEY_RETURN) {
        screen.push_back('\n');
    } else if (key == KEY_DELETE) {
        if (!screen.empty()) screen.pop_back();
    } else if (key == KEY_TAB) {
        screen.push_back('\t');
    } else {
        auto packed = key;
        if (flags & (kCGEventFlagMaskShift | kCGEventFlagMaskAlphaShift)) packed |= CAPS_MASK;
        auto character = keyCodeToCharacter(packed);
        // Space and other unshifted-only entries still have a physical key action.
        if (!character && packed != key) character = keyCodeToCharacter(key);
        if (character) screen.push_back(character);
    }
}

static constexpr UniChar rawKeySentinel = 0xffff;

// Link-local replacement shared by both implementations. No event leaves this process.
extern "C" void CGEventTapPostEvent(CGEventTapProxy, CGEventRef event) {
    ++posted;
    if (!recording || CGEventGetType(event) != kCGEventKeyDown)
        return;
    // Both paths use keycode zero for explicit Unicode payloads. Cached Backspace
    // and other virtual keys are interpreted from their keycodes and flags.
    if (CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) == 0) {
        UniChar text[32];
        UniCharCount count = 0;
        CGEventKeyboardGetUnicodeString(event, 32, &count, text);
        if (count > 32) std::abort();
        if (count && !(count == 1 && text[0] == rawKeySentinel)) {
            screen.insert(screen.end(), text, text + count);
            return;
        }
    }
    appendPhysical(event);
}

// The sandbox may deny private source creation and hang on a non-null source
// passed to the keyboard constructor. Keep real Quartz events and field operations.
static bool usePrivateSource;

extern "C" void bench_use_private_source(bool enabled) { usePrivateSource = enabled; }

extern "C" CGEventSourceRef bench_new_source() {
    if (usePrivateSource) return CGEventSourceCreate(kCGEventSourceStatePrivate);
    auto event = CGEventCreate(nullptr);
    if (!event) return nullptr;
    CGEventSetIntegerValueField(event, kCGEventSourceStateID, kCGEventSourceStateHIDSystemState);
    auto source = CGEventCreateSourceFromEvent(event);
    CFRelease(event);
    return source;
}

extern "C" CGEventRef CGEventCreateKeyboardEvent(CGEventSourceRef source, CGKeyCode key, bool down) {
    using Constructor = CGEventRef (*)(CGEventSourceRef, CGKeyCode, bool);
    static const auto create = reinterpret_cast<Constructor>(dlsym(RTLD_NEXT, "CGEventCreateKeyboardEvent"));
    if (!create) std::abort();
    auto event = create(usePrivateSource ? source : nullptr, key, down);
    if (recording && event && key == 0 && down) {
        // Quartz can populate a default Unicode value even for a physical key.
        // Mark raw key A so validation can distinguish it from an explicit payload.
        CGEventKeyboardSetUnicodeString(event, 1, &rawKeySentinel);
    }
    return event;
}

extern "C" void OpenKeyEnsureEventTap() {
    std::abort(); // The benchmark never installs a tap.
}

extern "C" void bench_callback_setup(bool step) {
    // The differential shim already reset the engine and installed the settings/macros.
    vSendKeyStepByStep = step;
    _lastFlag = 0;
    _hasJustUsedHotKey = false;
    _syncKey.clear();
    _cachedFrontPid = targetPid;
    _frontMostApp = @"org.neokey.bench.target";
    convertToolHotKey = EMPTY_HOTKEY;
    pData = static_cast<vKeyHookState*>(vKeyInit());
    if (!myEventSource) {
        myEventSource = bench_new_source();
        eventBackSpaceDown = CGEventCreateKeyboardEvent(myEventSource, KEY_DELETE, true);
        eventBackSpaceUp = CGEventCreateKeyboardEvent(myEventSource, KEY_DELETE, false);
        if (!myEventSource || !eventBackSpaceDown || !eventBackSpaceUp) {
            fprintf(stderr, "Could not create the callback source or cached Backspace events\n");
            std::abort();
        }
    }
}

extern "C" bool bench_callback(CGEventRef event) {
    @autoreleasepool {
        return OpenKeyCallback(nullptr, CGEventGetType(event), event, nullptr) != nullptr;
    }
}

extern "C" void bench_record(bool enabled) {
    recording = enabled;
    posted = 0;
    screen.clear();
}

extern "C" void bench_pass(CGEventRef event) { appendPhysical(event); }

extern "C" uint64_t bench_posted() { return posted; }

extern "C" const uint16_t* bench_screen(size_t* length) {
    *length = screen.size();
    return screen.data();
}
