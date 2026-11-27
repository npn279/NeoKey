#import <AppKit/AppKit.h>
#import <CoreGraphics/CoreGraphics.h>
#include <libproc.h>
#include <sys/resource.h>

static BOOL visibleWindow(pid_t pid) {
    NSArray *windows = CFBridgingRelease(CGWindowListCopyWindowInfo(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        kCGNullWindowID));
    for (NSDictionary *window in windows) {
        if ([window[(__bridge NSString *)kCGWindowOwnerPID] intValue] != pid)
            continue;
        CGRect bounds;
        if (CGRectMakeWithDictionaryRepresentation(
                (__bridge CFDictionaryRef)window[(__bridge NSString *)kCGWindowBounds], &bounds)
            && bounds.size.width >= 100 && bounds.size.height >= 40)
            return YES;
    }
    return NO;
}

// A negative result means Quartz did not disclose the tap list.
static int activeTaps(pid_t pid) {
    CGEventTapInformation taps[256];
    uint32_t count = 0;
    if (CGGetEventTapList(256, taps, &count) != kCGErrorSuccess || count >= 256)
        return -1;
    int matching = 0;
    for (uint32_t i = 0; i < count; ++i) {
        if (taps[i].tappingProcess == pid && taps[i].enabled
            && (taps[i].eventsOfInterest & CGEventMaskBit(kCGEventKeyDown)))
            ++matching;
    }
    return matching;
}

int main(void) {
    setvbuf(stdout, NULL, _IOLBF, 0);
    char line[1024];
    while (fgets(line, sizeof(line), stdin)) {
        @autoreleasepool {
            // NSRunningApplication refreshes changing properties on a common-mode
            // main-run-loop turn. The blocking command loop needs to supply one.
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.001, false);
            char bundle[512];
            if (sscanf(line, "existing %511s", bundle) == 1) {
                NSString *identifier = [NSString stringWithUTF8String:bundle];
                printf("%lu\n", (unsigned long)
                    [NSRunningApplication runningApplicationsWithBundleIdentifier:identifier].count);
                continue;
            }
            int pid, wantWindow, wantTaps;
            if (sscanf(line, "pid %d %d %d", &pid, &wantWindow, &wantTaps) != 3 || pid < 1)
                return 2;

            struct rusage_info_v4 usage = {0};
            struct proc_taskinfo task = {0};
            if (proc_pid_rusage(pid, RUSAGE_INFO_V4, (rusage_info_t *)&usage) != 0
                || proc_pidinfo(pid, PROC_PIDTASKINFO, 0, &task, sizeof(task)) != sizeof(task)) {
                puts("unavailable");
                continue;
            }
            NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:pid];
            printf("%d %d %d %llu %llu %d %d\n", app != nil && app.bundleIdentifier != nil,
                   app.finishedLaunching, wantWindow && app && visibleWindow(pid),
                   (unsigned long long)(usage.ri_user_time + usage.ri_system_time),
                   (unsigned long long)usage.ri_phys_footprint, task.pti_threadnum,
                   wantTaps ? activeTaps(pid) : -1);
        }
    }
}
