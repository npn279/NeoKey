// Open Spotlight, type "oo" through the event tap, and check the AX field.
#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#include <stdio.h>
#include <unistd.h>

static void key(CGKeyCode code, CGEventFlags flags) {
    CGEventSourceRef source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
    for (int down = 1; down >= 0; down--) {
        CGEventRef event = CGEventCreateKeyboardEvent(source, code, down);
        CGEventSetFlags(event, flags);
        CGEventPost(kCGHIDEventTap, event);
        CFRelease(event);
        usleep(30000);
    }
    CFRelease(source);
}

static NSString *focusedValue(pid_t *pidOut) {
    AXUIElementRef system = AXUIElementCreateSystemWide();
    AXUIElementSetMessagingTimeout(system, 0.08f);
    AXUIElementRef focused = NULL;
    NSString *text = nil;
    if (AXUIElementCopyAttributeValue(system, kAXFocusedUIElementAttribute,
                                      (CFTypeRef *)&focused) == kAXErrorSuccess && focused != NULL) {
        AXUIElementGetPid(focused, pidOut);
        CFTypeRef value = NULL;
        if (AXUIElementCopyAttributeValue(focused, kAXValueAttribute, &value) == kAXErrorSuccess &&
            value != NULL && CFGetTypeID(value) == CFStringGetTypeID()) {
            text = (__bridge NSString *)value;
        }
        if (value != NULL)
            CFRelease(value);
        CFRelease(focused);
    }
    CFRelease(system);
    return text;
}

int main(void) {
    @autoreleasepool {
        key(49, kCGEventFlagMaskCommand); // Cmd+Space
        usleep(900000);
        key(31, 0); // o
        usleep(150000);
        key(31, 0); // o
        usleep(400000);

        pid_t pid = 0;
        NSString *value = focusedValue(&pid);
        NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:pid];
        printf("FOCUS_BUNDLE=%s\nRESULT=%s\n", app.bundleIdentifier.UTF8String ?: "(unknown)",
               value.UTF8String ?: "(no AX value)");

        key(53, 0); // Escape clears the field
        usleep(150000);
        key(53, 0); // Escape closes Spotlight
        return [value isEqualToString:@"ô"] ? 0 : 1;
    }
}
