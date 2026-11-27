//
//  OpenKey.m
//  OpenKey
//
//  Created by Tuyen on 1/18/19.
//  Copyright © 2019 Tuyen Mai. All rights reserved.
//
#import <Cocoa/Cocoa.h>
#import <Carbon/Carbon.h>
#import <Foundation/Foundation.h>
#import <ApplicationServices/ApplicationServices.h>
#import "Engine.h"
#import "AppDelegate.h"
#import "ViewController.h"

/* Cached front app. Do not call NSWorkspace on the key path. */
#define FRONT_APP _frontMostApp
#define OTHER_CONTROL_KEY (_flag & kCGEventFlagMaskCommand) || (_flag & kCGEventFlagMaskControl) || \
                            (_flag & kCGEventFlagMaskAlternate) || (_flag & kCGEventFlagMaskSecondaryFn) || \
                            (_flag & kCGEventFlagMaskNumericPad) || (_flag & kCGEventFlagMaskHelp)

#define DYNA_DATA(macro, pos) (macro ? pData->macroData[pos] : pData->charData[pos])
#define MAX_UNICODE_STRING  20
#define EMPTY_HOTKEY 0xFE0000FE
#define LOAD_DATA(VAR, KEY) VAR = (int)[[NSUserDefaults standardUserDefaults] integerForKey:@#KEY]

// Ignore code for Modifier keys and numpad
// Reference: https://eastmanreference.com/complete-list-of-applescript-key-codes
NSDictionary *keyStringToKeyCodeMap = @{
    // Characters from number row
    @"`": @50, @"~": @50, @"1": @18, @"!": @18, @"2": @19, @"@": @19, @"3": @20, @"#": @20, @"4": @21, @"$": @21,
    @"5": @23, @"%": @23, @"6": @22, @"^": @22, @"7": @26, @"&": @26, @"8": @28, @"*": @28, @"9": @25, @"(": @25,
    @"0": @29, @")": @29, @"-": @27, @"_": @27, @"=": @24, @"+": @24,
    // Characters from first keyboard row
    @"q": @12, @"w": @13, @"e": @14, @"r": @15, @"t": @17, @"y": @16, @"u": @32, @"i": @34, @"o": @31, @"p": @35,
    @"[": @33, @"{": @33, @"]": @30, @"}": @30, @"\\": @42, @"|": @42,
    // Characters from second keyboard row
    @"a": @0, @"s": @1, @"d": @2, @"f": @3, @"g": @5, @"h": @4, @"j": @38, @"k": @40, @"l": @37,
    @";": @41, @":": @41, @"'": @39, @"\"": @39,
    // Characters from second third row
    @"z": @6, @"x": @7, @"c": @8, @"v": @9, @"b": @11, @"n": @45, @"m": @46,
    @",": @43, @"<": @43, @".": @47, @">": @47, @"/": @44, @"?": @44
};

extern ViewController* viewController;

extern AppDelegate* appDelegate;
extern int vSendKeyStepByStep;
extern int vFixChromiumBrowser;
extern int vPerformLayoutCompat;

extern "C" {
    //app which must sent special empty character
    NSArray* _niceSpaceApp = @[@"com.sublimetext.3",
                               @"com.sublimetext.2",
                             ];
    
    //app which error with unicode Compound
    NSArray* _unicodeCompoundApp = @[@"com.apple.",
                                     @"com.google.Chrome", @"com.brave.Browser",
                                     @"com.microsoft.edgemac.Dev", @"com.microsoft.edgemac.Beta", @"com.microsoft.Edge.Dev", @"com.microsoft.Edge"];
    NSArray* _recommendWorkaroundDisabledApp = @[@"com.apple.Spotlight"];
    
    CGEventSourceRef myEventSource = NULL;
    vKeyHookState* pData;
    CGEventRef eventBackSpaceDown;
    CGEventRef eventBackSpaceUp;
    UniChar _newChar, _newCharHi;
    CGEventRef _newEventDown, _newEventUp;
    CGKeyCode _keycode;
    CGEventFlags _flag, _lastFlag = 0, _privateFlag;
    CGEventTapProxy _proxy;
    
    Uint16 _newCharString[MAX_UNICODE_STRING];
    Uint16 _newCharSize;
    bool _willContinuteSending = false;
    bool _willSendControlKey = false;
    
    vector<Uint16> _syncKey;
    
    Uint16 _uniChar[2];
    int _i, _j, _k;
    Uint32 _tempChar;
    bool _hasJustUsedHotKey = false;

    int _languageTemp = 0; //use for smart switch key
    vector<Byte> savedSmartSwitchKeyData; ////use for smart switch key
    
    NSString* _frontMostApp = @"UnknownApp";
    pid_t _cachedFrontPid = -1;
    NSMutableDictionary *_bundleIdByPid = nil;

    void queryFrontMostApp(void);
    void refreshFrontAppFromEvent(CGEventRef event);
    void OpenKeyEnsureEventTap(void);
    static bool commitLauncherComposition(void);
    
    void OpenKeyInit() {
        //load saved data
        vFreeMark = 0;//(int)[[NSUserDefaults standardUserDefaults] integerForKey:@"FreeMark"];
        LOAD_DATA(vCodeTable, CodeTable); if (vCodeTable < 0) vCodeTable = 0;
        LOAD_DATA(vCheckSpelling, Spelling);
        LOAD_DATA(vQuickTelex, QuickTelex);
        LOAD_DATA(vUseModernOrthography, ModernOrthography);
        LOAD_DATA(vRestoreIfWrongSpelling, RestoreIfInvalidWord);
        LOAD_DATA(vFixRecommendBrowser, FixRecommendBrowser);
        LOAD_DATA(vUseMacro, UseMacro);
        LOAD_DATA(vUseMacroInEnglishMode, UseMacroInEnglishMode);
        LOAD_DATA(vAutoCapsMacro, vAutoCapsMacro);
        LOAD_DATA(vSendKeyStepByStep, SendKeyStepByStep);
        LOAD_DATA(vUseSmartSwitchKey, UseSmartSwitchKey);
        LOAD_DATA(vUpperCaseFirstChar, UpperCaseFirstChar);
        
        LOAD_DATA(vTempOffSpelling, vTempOffSpelling);
        LOAD_DATA(vAllowConsonantZFWJ, vAllowConsonantZFWJ);
        LOAD_DATA(vQuickEndConsonant, vQuickEndConsonant);
        LOAD_DATA(vQuickStartConsonant, vQuickStartConsonant);
        LOAD_DATA(vRememberCode, vRememberCode);
        LOAD_DATA(vOtherLanguage, vOtherLanguage);
        LOAD_DATA(vTempOffOpenKey, vTempOffOpenKey);
        
        LOAD_DATA(vFixChromiumBrowser, vFixChromiumBrowser);
        
        LOAD_DATA(vPerformLayoutCompat, vPerformLayoutCompat);
        
        myEventSource = CGEventSourceCreate(kCGEventSourceStatePrivate);
        pData = (vKeyHookState*)vKeyInit();

        eventBackSpaceDown = CGEventCreateKeyboardEvent (myEventSource, 51, true);
        eventBackSpaceUp = CGEventCreateKeyboardEvent (myEventSource, 51, false);
        
        //init and load macro data
        NSUserDefaults *prefs = [NSUserDefaults standardUserDefaults];
        NSData *data = [prefs objectForKey:@"macroData"];
        initMacroMap((Byte*)data.bytes, (int)data.length);
        
        //init and load smart switch key data
        data = [prefs objectForKey:@"smartSwitchKey"];
        initSmartSwitchKey((Byte*)data.bytes, (int)data.length);
        
        //init convert tool
        convertToolDontAlertWhenCompleted = ![prefs boolForKey:@"convertToolDontAlertWhenCompleted"];
        convertToolToAllCaps = [prefs boolForKey:@"convertToolToAllCaps"];
        convertToolToAllNonCaps = [prefs boolForKey:@"convertToolToAllNonCaps"];
        convertToolToCapsFirstLetter = [prefs boolForKey:@"convertToolToCapsFirstLetter"];
        convertToolToCapsEachWord = [prefs boolForKey:@"convertToolToCapsEachWord"];
        convertToolRemoveMark = [prefs boolForKey:@"convertToolRemoveMark"];
        convertToolFromCode = [prefs integerForKey:@"convertToolFromCode"];
        convertToolToCode = [prefs integerForKey:@"convertToolToCode"];
        convertToolHotKey = (int)[prefs integerForKey:@"convertToolHotKey"];
        if (convertToolHotKey == 0) {
            convertToolHotKey = EMPTY_HOTKEY;
        }
        queryFrontMostApp();
    }
    
    void RequestNewSession() {
        //send event signal to Engine
        vKeyHandleEvent(vKeyEvent::Mouse, vKeyEventState::MouseDown, 0);
        
        if (IS_DOUBLE_CODE(vCodeTable)) { //VNI
            _syncKey.clear();
        }
    }
    
    static void rememberFrontApp(NSRunningApplication *app) {
        if (app == nil)
            return;
        NSString *bid = app.bundleIdentifier;
        if (bid == nil)
            bid = app.localizedName != nil ? app.localizedName : @"UnknownApp";
        pid_t pid = app.processIdentifier;
        if (_bundleIdByPid == nil)
            _bundleIdByPid = [[NSMutableDictionary alloc] init];
        if (pid > 0) {
            if (_bundleIdByPid.count > 128)
                [_bundleIdByPid removeAllObjects];
            _bundleIdByPid[@(pid)] = bid;
        }
        if ([bid compare:OPENKEY_BUNDLE] == 0)
            return;
        _frontMostApp = bid;
        if (pid > 0)
            _cachedFrontPid = pid;
    }

    void queryFrontMostApp() {
        rememberFrontApp([[NSWorkspace sharedWorkspace] frontmostApplication]);
    }

    void refreshFrontAppFromEvent(CGEventRef event) {
        if (event == NULL)
            return;
        pid_t pid = (pid_t)CGEventGetIntegerValueField(event, kCGEventTargetUnixProcessID);
        if (pid <= 0)
            return;
        if (pid == _cachedFrontPid && _frontMostApp != nil)
            return;
        NSString *cached = _bundleIdByPid[@(pid)];
        if (cached != nil) {
            if ([cached compare:OPENKEY_BUNDLE] != 0) {
                _frontMostApp = cached;
                _cachedFrontPid = pid;
            }
            return;
        }
        rememberFrontApp([NSRunningApplication runningApplicationWithProcessIdentifier:pid]);
    }
    
    NSString* ConvertUtil(NSString* str) {
        return [NSString stringWithUTF8String:convertUtil([str UTF8String]).c_str()];
    }
    
    BOOL containUnicodeCompoundApp(NSString* topApp) {
        if (topApp == nil) return false;
        for (_j = 0; _j < [_unicodeCompoundApp count]; _j++) {
            if ([topApp hasPrefix:[_unicodeCompoundApp objectAtIndex:_j]] || [[_unicodeCompoundApp objectAtIndex:_j] isEqualToString:topApp])
                return true;
        }
        return false;
    }

    BOOL isLauncherSearchApp(NSString* topApp) {
        if (topApp == nil) return false;
        if ([topApp isEqualToString:@"com.apple.Spotlight"] ||
            [topApp hasPrefix:@"com.apple.Spotlight."] ||
            [topApp isEqualToString:@"com.runningwithcrayons.Alfred"] ||
            [topApp isEqualToString:@"com.runningwithcrayons.Alfred-3"] ||
            [topApp hasPrefix:@"com.runningwithcrayons.Alfred."] ||
            [topApp isEqualToString:@"com.raycast.macos"] ||
            [topApp hasPrefix:@"com.raycast.macos."] ||
            [topApp isEqualToString:@"Spotlight"] ||
            [topApp isEqualToString:@"Alfred"] ||
            [topApp isEqualToString:@"Raycast"]) {
            return true;
        }
        return false;
    }

    BOOL shouldUseRecommendWorkaround(NSString* topApp) {
        if (!vFixRecommendBrowser) return false;
        if (isLauncherSearchApp(topApp)) return false;
        if (topApp == nil) return true;
        return ![_recommendWorkaroundDisabledApp containsObject:topApp];
    }

    BOOL shouldUseSelectionReplacement(NSString* topApp) {
        return isLauncherSearchApp(topApp);
    }
    
    void saveSmartSwitchKeyData() {
        getSmartSwitchKeySaveData(savedSmartSwitchKeyData);
        NSData* _data = [NSData dataWithBytes:savedSmartSwitchKeyData.data() length:savedSmartSwitchKeyData.size()];
        NSUserDefaults *prefs = [NSUserDefaults standardUserDefaults];
        [prefs setObject:_data forKey:@"smartSwitchKey"];
    }
    
    void OnActiveAppChanged() { //use for smart switch key; improved on Sep 28th, 2019
        queryFrontMostApp();
        _languageTemp = getAppInputMethodStatus(string(_frontMostApp.UTF8String), vLanguage | (vCodeTable << 1));
        if ((_languageTemp & 0x01) != vLanguage) { //for input method
            if (_languageTemp != -1) {
                vLanguage = _languageTemp;
                [appDelegate onImputMethodChanged:NO];
                startNewSession();
            } else {
                saveSmartSwitchKeyData();
            }
        }
        if (vRememberCode && (_languageTemp >> 1) != vCodeTable) { //for remember table code feature
            if (_languageTemp != -1) {
                [appDelegate onCodeTableChanged:(_languageTemp >> 1)];
            } else {
                saveSmartSwitchKeyData();
            }
        }
    }
    
    void OnTableCodeChange() {
        onTableCodeChange();
        if (vRememberCode) {
            queryFrontMostApp();
            setAppInputMethodStatus(string(_frontMostApp.UTF8String), vLanguage | (vCodeTable << 1));
            saveSmartSwitchKeyData();
        }
    }
    
    void OnInputMethodChanged() {
        if (vUseSmartSwitchKey) {
            queryFrontMostApp();
            setAppInputMethodStatus(string(_frontMostApp.UTF8String), vLanguage | (vCodeTable << 1));
            saveSmartSwitchKeyData();
        }
    }
    
    void OnSpellCheckingChanged() {
        vSetCheckSpelling();
    }
    
    void InsertKeyLength(const Uint8& len) {
        _syncKey.push_back(len);
    }
    
    void SendPureCharacter(const Uint16& ch) {
        _newEventDown = CGEventCreateKeyboardEvent(myEventSource, 0, true);
        _newEventUp = CGEventCreateKeyboardEvent(myEventSource, 0, false);
        CGEventKeyboardSetUnicodeString(_newEventDown, 1, &ch);
        CGEventKeyboardSetUnicodeString(_newEventUp, 1, &ch);
        CGEventTapPostEvent(_proxy, _newEventDown);
        CGEventTapPostEvent(_proxy, _newEventUp);
        CFRelease(_newEventDown);
        CFRelease(_newEventUp);
        if (IS_DOUBLE_CODE(vCodeTable)) {
            InsertKeyLength(1);
        }
    }
    
    void SendKeyCode(Uint32 data) {
        _newChar = (Uint16)data;
        if (!(data & CHAR_CODE_MASK)) {
            if (IS_DOUBLE_CODE(vCodeTable)) //VNI
                InsertKeyLength(1);
            
            _newEventDown = CGEventCreateKeyboardEvent(myEventSource, _newChar, true);
            _newEventUp = CGEventCreateKeyboardEvent(myEventSource, _newChar, false);
            _privateFlag = CGEventGetFlags(_newEventDown);
            
            if (data & CAPS_MASK) {
                _privateFlag |= kCGEventFlagMaskShift;
            } else {
                _privateFlag &= ~kCGEventFlagMaskShift;
            }
            _privateFlag |= kCGEventFlagMaskNonCoalesced;
            
            CGEventSetFlags(_newEventDown, _privateFlag);
            CGEventSetFlags(_newEventUp, _privateFlag);
            CGEventTapPostEvent(_proxy, _newEventDown);
            CGEventTapPostEvent(_proxy, _newEventUp);
        } else {
            if (vCodeTable == 0) { //unicode 2 bytes code
                _newEventDown = CGEventCreateKeyboardEvent(myEventSource, 0, true);
                _newEventUp = CGEventCreateKeyboardEvent(myEventSource, 0, false);
                CGEventKeyboardSetUnicodeString(_newEventDown, 1, &_newChar);
                CGEventKeyboardSetUnicodeString(_newEventUp, 1, &_newChar);
                CGEventTapPostEvent(_proxy, _newEventDown);
                CGEventTapPostEvent(_proxy, _newEventUp);
            } else if (vCodeTable == 1 || vCodeTable == 2 || vCodeTable == 4) { //others such as VNI Windows, TCVN3: 1 byte code
                _newCharHi = HIBYTE(_newChar);
                _newChar = LOBYTE(_newChar);
                
                _newEventDown = CGEventCreateKeyboardEvent(myEventSource, 0, true);
                _newEventUp = CGEventCreateKeyboardEvent(myEventSource, 0, false);
                CGEventKeyboardSetUnicodeString(_newEventDown, 1, &_newChar);
                CGEventKeyboardSetUnicodeString(_newEventUp, 1, &_newChar);
                CGEventTapPostEvent(_proxy, _newEventDown);
                CGEventTapPostEvent(_proxy, _newEventUp);
                if (_newCharHi > 32) {
                    if (vCodeTable == 2) //VNI
                        InsertKeyLength(2);
                    CFRelease(_newEventDown);
                    CFRelease(_newEventUp);
                    _newEventDown = CGEventCreateKeyboardEvent(myEventSource, 0, true);
                    _newEventUp = CGEventCreateKeyboardEvent(myEventSource, 0, false);
                    CGEventKeyboardSetUnicodeString(_newEventDown, 1, &_newCharHi);
                    CGEventKeyboardSetUnicodeString(_newEventUp, 1, &_newCharHi);
                    CGEventTapPostEvent(_proxy, _newEventDown);
                    CGEventTapPostEvent(_proxy, _newEventUp);
                } else {
                    if (vCodeTable == 2) //VNI
                        InsertKeyLength(1);
                }
            } else if (vCodeTable == 3) { //Unicode Compound
                _newCharHi = (_newChar >> 13);
                _newChar &= 0x1FFF;
                _uniChar[0] = _newChar;
                _uniChar[1] = _newCharHi > 0 ? (_unicodeCompoundMark[_newCharHi - 1]) : 0;
                InsertKeyLength(_newCharHi > 0 ? 2 : 1);
                _newEventDown = CGEventCreateKeyboardEvent(myEventSource, 0, true);
                _newEventUp = CGEventCreateKeyboardEvent(myEventSource, 0, false);
                CGEventKeyboardSetUnicodeString(_newEventDown, (_newCharHi > 0 ? 2 : 1), _uniChar);
                CGEventKeyboardSetUnicodeString(_newEventUp, (_newCharHi > 0 ? 2 : 1), _uniChar);
                CGEventTapPostEvent(_proxy, _newEventDown);
                CGEventTapPostEvent(_proxy, _newEventUp);
            }
        }
        CFRelease(_newEventDown);
        CFRelease(_newEventUp);
    }
    
    void SendEmptyCharacter() {
        if (IS_DOUBLE_CODE(vCodeTable)) //VNI or Unicode Compound
            InsertKeyLength(1);
        
        _newChar = 0x202F; //empty char
        if ([_niceSpaceApp containsObject:FRONT_APP]) {
            _newChar = 0x200C; //Unicode character with empty space
        }
        
        _newEventDown = CGEventCreateKeyboardEvent(myEventSource, 0, true);
        _newEventUp = CGEventCreateKeyboardEvent(myEventSource, 0, false);
        CGEventKeyboardSetUnicodeString(_newEventDown, 1, &_newChar);
        CGEventKeyboardSetUnicodeString(_newEventUp, 1, &_newChar);
        CGEventTapPostEvent(_proxy, _newEventDown);
        CGEventTapPostEvent(_proxy, _newEventUp);
        CFRelease(_newEventDown);
        CFRelease(_newEventUp);
    }
    
    void SendVirtualKey(const Byte& vKey) {
        CGEventRef eventVkeyDown = CGEventCreateKeyboardEvent (myEventSource, vKey, true);
        CGEventRef eventVkeyUp = CGEventCreateKeyboardEvent (myEventSource, vKey, false);
        
        CGEventTapPostEvent(_proxy, eventVkeyDown);
        CGEventTapPostEvent(_proxy, eventVkeyUp);
        
        CFRelease(eventVkeyDown);
        CFRelease(eventVkeyUp);
    }

    void SendBackspace() {
        CGEventTapPostEvent(_proxy, eventBackSpaceDown);
        CGEventTapPostEvent(_proxy, eventBackSpaceUp);
        
        if (IS_DOUBLE_CODE(vCodeTable)) { //VNI or Unicode Compound
            if (_syncKey.back() > 1) {
                if (!(vCodeTable == 3 && containUnicodeCompoundApp(FRONT_APP))) {
                    CGEventTapPostEvent(_proxy, eventBackSpaceDown);
                    CGEventTapPostEvent(_proxy, eventBackSpaceUp);
                }
            }
            _syncKey.pop_back();
        }
    }
    
    void SendShiftAndLeftArrow() {
        CGEventRef eventVkeyDown = CGEventCreateKeyboardEvent (myEventSource, KEY_LEFT, true);
        CGEventRef eventVkeyUp = CGEventCreateKeyboardEvent (myEventSource, KEY_LEFT, false);
        _privateFlag = CGEventGetFlags(eventVkeyDown);
        _privateFlag |= kCGEventFlagMaskShift;
        CGEventSetFlags(eventVkeyDown, _privateFlag);
        CGEventSetFlags(eventVkeyUp, _privateFlag);
        
        CGEventTapPostEvent(_proxy, eventVkeyDown);
        CGEventTapPostEvent(_proxy, eventVkeyUp);
        
        if (IS_DOUBLE_CODE(vCodeTable)) { //VNI or Unicode Compound
            if (_syncKey.back() > 1) {
                if (!(vCodeTable == 3 && containUnicodeCompoundApp(FRONT_APP))) {
                    CGEventTapPostEvent(_proxy, eventVkeyDown);
                    CGEventTapPostEvent(_proxy, eventVkeyUp);
                }
            }
            _syncKey.pop_back();
        }
        CFRelease(eventVkeyDown);
        CFRelease(eventVkeyUp);
    }
    
    void SendCutKey() {
        CGEventRef eventVkeyDown = CGEventCreateKeyboardEvent (myEventSource, KEY_X, true);
        CGEventRef eventVkeyUp = CGEventCreateKeyboardEvent (myEventSource, KEY_X, false);
        _privateFlag = CGEventGetFlags(eventVkeyDown);
        _privateFlag |= NX_COMMANDMASK;
        CGEventSetFlags(eventVkeyDown, _privateFlag);
        CGEventSetFlags(eventVkeyUp, _privateFlag);
        
        CGEventTapPostEvent(_proxy, eventVkeyDown);
        CGEventTapPostEvent(_proxy, eventVkeyUp);
        
        CFRelease(eventVkeyDown);
        CFRelease(eventVkeyUp);
    }
    
    void SendNewCharString(const bool& dataFromMacro=false, const Uint16& offset=0) {
        _j = 0;
        _newCharSize = dataFromMacro ? pData->macroData.size() : pData->newCharCount;
        _willContinuteSending = false;
        _willSendControlKey = false;
        
        if (_newCharSize > 0) {
            for (_k = dataFromMacro ? offset : pData->newCharCount - 1 - offset;
                 dataFromMacro ? _k < pData->macroData.size() : _k >= 0;
                 dataFromMacro ? _k++ : _k--) {
                
                if (_j >= 16) {
                    _willContinuteSending = true;
                    break;
                }
                
                _tempChar = DYNA_DATA(dataFromMacro, _k);
                if (_tempChar & PURE_CHARACTER_MASK) {
                    _newCharString[_j++] = _tempChar;
                    if (IS_DOUBLE_CODE(vCodeTable)) {
                        InsertKeyLength(1);
                    }
                } else if (!(_tempChar & CHAR_CODE_MASK)) {
                    if (IS_DOUBLE_CODE(vCodeTable)) //VNI
                        InsertKeyLength(1);
                    _newCharString[_j++] = keyCodeToCharacter(_tempChar);
                } else {
                    if (vCodeTable == 0) {  //unicode 2 bytes code
                        _newCharString[_j++] = _tempChar;
                    } else if (vCodeTable == 1 || vCodeTable == 2 || vCodeTable == 4) { //others such as VNI Windows, TCVN3: 1 byte code
                        _newChar = _tempChar;
                        _newCharHi = HIBYTE(_newChar);
                        _newChar = LOBYTE(_newChar);
                        _newCharString[_j++] = _newChar;
                        
                        if (_newCharHi > 32) {
                            if (vCodeTable == 2) //VNI
                                InsertKeyLength(2);
                            _newCharString[_j++] = _newCharHi;
                            _newCharSize++;
                        } else {
                            if (vCodeTable == 2) //VNI
                                InsertKeyLength(1);
                        }
                    } else if (vCodeTable == 3) { //Unicode Compound
                        _newChar = _tempChar;
                        _newCharHi = (_newChar >> 13);
                        _newChar &= 0x1FFF;
                        
                        InsertKeyLength(_newCharHi > 0 ? 2 : 1);
                        _newCharString[_j++] = _newChar;
                        if (_newCharHi > 0) {
                            _newCharSize++;
                            _newCharString[_j++] = _unicodeCompoundMark[_newCharHi - 1];
                        }
                        
                    }
                }
            }//end for
        }
        
        if (!_willContinuteSending && (pData->code == vRestore || pData->code == vRestoreAndStartNewSession)) { //if is restore
            if (keyCodeToCharacter(_keycode) != 0) {
                _newCharSize++;
                _newCharString[_j++] = keyCodeToCharacter(_keycode | ((_flag & kCGEventFlagMaskAlphaShift) || (_flag & kCGEventFlagMaskShift) ? CAPS_MASK : 0));
            } else {
                _willSendControlKey = true;
            }
        }
        if (!_willContinuteSending && pData->code == vRestoreAndStartNewSession) {
            startNewSession();
        }
        
        _newEventDown = CGEventCreateKeyboardEvent(myEventSource, 0, true);
        _newEventUp = CGEventCreateKeyboardEvent(myEventSource, 0, false);
        CGEventKeyboardSetUnicodeString(_newEventDown, _willContinuteSending ? 16 : _newCharSize - offset, _newCharString);
        CGEventKeyboardSetUnicodeString(_newEventUp, _willContinuteSending ? 16 : _newCharSize - offset, _newCharString);
        CGEventTapPostEvent(_proxy, _newEventDown);
        CGEventTapPostEvent(_proxy, _newEventUp);
        CFRelease(_newEventDown);
        CFRelease(_newEventUp);

        if (_willContinuteSending) {
            SendNewCharString(dataFromMacro, dataFromMacro ? _k : 16);
        }
        
        //the case when hCode is vRestore or vRestoreAndStartNewSession, the word is invalid and last key is control key such as TAB, LEFT ARROW, RIGHT ARROW,...
        if (_willSendControlKey) {
            SendKeyCode(_keycode);
        }
    }
            
    bool checkHotKey(int hotKeyData, bool checkKeyCode=true) {
        if ((hotKeyData & (~0x8000)) == EMPTY_HOTKEY)
            return false;
        if (HAS_CONTROL(hotKeyData) ^ GET_BOOL(_lastFlag & kCGEventFlagMaskControl))
            return false;
        if (HAS_OPTION(hotKeyData) ^ GET_BOOL(_lastFlag & kCGEventFlagMaskAlternate))
            return false;
        if (HAS_COMMAND(hotKeyData) ^ GET_BOOL(_lastFlag & kCGEventFlagMaskCommand))
            return false;
        if (HAS_SHIFT(hotKeyData) ^ GET_BOOL(_lastFlag & kCGEventFlagMaskShift))
            return false;
        if (checkKeyCode) {
            if (GET_SWITCH_KEY(hotKeyData) != _keycode)
                return false;
        }
        return true;
    }
    
    void switchLanguage() {
        if (vLanguage == 0)
            vLanguage = 1;
        else
            vLanguage = 0;
        if (HAS_BEEP(vSwitchKeyStatus))
            NSBeep();
        [appDelegate onImputMethodChanged:YES];
        startNewSession();
    }
    
    /* UTF-16 composition replace. Same arithmetic as tests/composition_replace.c. */
    static NSString* stringByReplacingComposition(NSString* value, NSUInteger caret, NSUInteger selectionLength, NSUInteger deleteCount, NSString* replacement, NSUInteger* outCaret) {
        if (value == nil)
            value = @"";
        if (replacement == nil)
            replacement = @"";
        if (caret > value.length)
            caret = value.length;
        if (selectionLength > value.length - caret)
            selectionLength = value.length - caret;
        if (deleteCount > caret)
            deleteCount = caret;
        NSUInteger start = caret - deleteCount;
        NSUInteger replaceLen = deleteCount + selectionLength;
        NSString* updated = [value stringByReplacingCharactersInRange:NSMakeRange(start, replaceLen) withString:replacement];
        if (outCaret != NULL)
            *outCaret = start + replacement.length;
        return updated;
    }

    static void appendOutgoingUnit(UniChar* buf, int cap, int* len, vector<Uint8>* insertLens, Uint32 tempChar) {
        if (*len >= cap)
            return;
        if (tempChar & PURE_CHARACTER_MASK) {
            buf[(*len)++] = (UniChar)tempChar;
            if (insertLens != NULL)
                insertLens->push_back(1);
            return;
        }
        if (!(tempChar & CHAR_CODE_MASK)) {
            if (insertLens != NULL)
                insertLens->push_back(1);
            if (*len < cap)
                buf[(*len)++] = keyCodeToCharacter(tempChar);
            return;
        }
        if (vCodeTable == 0) {
            buf[(*len)++] = (UniChar)tempChar;
            return;
        }
        if (vCodeTable == 1 || vCodeTable == 2 || vCodeTable == 4) {
            UniChar lo = (UniChar)LOBYTE(tempChar);
            UniChar hi = (UniChar)HIBYTE(tempChar);
            buf[(*len)++] = lo;
            if (hi > 32) {
                if (vCodeTable == 2 && insertLens != NULL)
                    insertLens->push_back(2);
                if (*len < cap)
                    buf[(*len)++] = hi;
            } else if (vCodeTable == 2 && insertLens != NULL) {
                insertLens->push_back(1);
            }
            return;
        }
        if (vCodeTable == 3) {
            UniChar ch = (UniChar)(tempChar & 0x1FFF);
            UniChar mark = (UniChar)(tempChar >> 13);
            if (insertLens != NULL)
                insertLens->push_back(mark > 0 ? 2 : 1);
            buf[(*len)++] = ch;
            if (mark > 0 && *len < cap)
                buf[(*len)++] = _unicodeCompoundMark[mark - 1];
        }
    }

    static bool collectOutgoing(UniChar* buf, int cap, int* outLen, bool* sendControl, vector<Uint8>* insertLens) {
        *outLen = 0;
        *sendControl = false;
        if (insertLens != NULL)
            insertLens->clear();
        if (pData->newCharCount > 0) {
            for (int k = pData->newCharCount - 1; k >= 0; k--) {
                if (*outLen >= cap)
                    return false;
                int before = *outLen;
                appendOutgoingUnit(buf, cap, outLen, insertLens, pData->charData[k]);
                if (*outLen == before && (pData->charData[k] & (PURE_CHARACTER_MASK | CHAR_CODE_MASK)))
                    return false;
                if (*outLen >= cap && k > 0)
                    return false;
            }
        }
        if (pData->code == vRestore || pData->code == vRestoreAndStartNewSession) {
            Uint16 ch = keyCodeToCharacter(_keycode | (((_flag & kCGEventFlagMaskAlphaShift) || (_flag & kCGEventFlagMaskShift)) ? CAPS_MASK : 0));
            if (ch != 0) {
                if (*outLen >= cap)
                    return false;
                buf[(*outLen)++] = ch;
            } else {
                *sendControl = true;
            }
        }
        return true;
    }

    static bool replaceFocusedComposition(NSString* replacement, NSUInteger deleteCount) {
        AXUIElementRef systemWide = AXUIElementCreateSystemWide();
        if (systemWide == NULL)
            return false;
        AXUIElementRef focused = NULL;
        AXError err = AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute, (CFTypeRef*)&focused);
        CFRelease(systemWide);
        if (err != kAXErrorSuccess || focused == NULL)
            return false;
        AXUIElementSetMessagingTimeout(focused, 0.08f);

        CFTypeRef valueRef = NULL;
        err = AXUIElementCopyAttributeValue(focused, kAXValueAttribute, &valueRef);
        if (err != kAXErrorSuccess || valueRef == NULL || CFGetTypeID(valueRef) != CFStringGetTypeID()) {
            if (valueRef != NULL)
                CFRelease(valueRef);
            CFRelease(focused);
            return false;
        }
        NSString* value = [NSString stringWithString:(__bridge NSString*)valueRef];
        CFRelease(valueRef);

        NSRange sel = NSMakeRange(value.length, 0);
        CFTypeRef rangeRef = NULL;
        if (AXUIElementCopyAttributeValue(focused, kAXSelectedTextRangeAttribute, &rangeRef) == kAXErrorSuccess && rangeRef != NULL) {
            if (CFGetTypeID(rangeRef) == AXValueGetTypeID()) {
                CFRange cf = {0, 0};
                if (AXValueGetValue((AXValueRef)rangeRef, kAXValueTypeCFRange, &cf) && cf.location >= 0 && cf.length >= 0) {
                    sel.location = (NSUInteger)cf.location;
                    sel.length = (NSUInteger)cf.length;
                }
            }
            CFRelease(rangeRef);
        }

        NSUInteger newCaret = 0;
        NSString* updated = stringByReplacingComposition(value, sel.location, sel.length, deleteCount, replacement, &newCaret);
        err = AXUIElementSetAttributeValue(focused, kAXValueAttribute, (__bridge CFTypeRef)updated);
        if (err == kAXErrorSuccess) {
            CFRange caret = CFRangeMake((CFIndex)newCaret, 0);
            AXValueRef axCaret = AXValueCreate(kAXValueTypeCFRange, &caret);
            if (axCaret != NULL) {
                AXUIElementSetAttributeValue(focused, kAXSelectedTextRangeAttribute, axCaret);
                CFRelease(axCaret);
            }
        }
        CFRelease(focused);
        return err == kAXErrorSuccess;
    }

    static bool commitLauncherComposition(void) {
        UniChar chars[MAX_BUFF * 2];
        int len = 0;
        bool sendControl = false;
        vector<Uint8> inserts;
        vector<Uint8>* insertLens = IS_DOUBLE_CODE(vCodeTable) ? &inserts : NULL;
        if (!collectOutgoing(chars, MAX_BUFF * 2, &len, &sendControl, insertLens))
            return false;
        if (len == 0 && pData->backspaceCount == 0 && !sendControl)
            return false;
        NSString* replacement = [NSString stringWithCharacters:chars length:(NSUInteger)len];
        if (!replaceFocusedComposition(replacement, pData->backspaceCount))
            return false;
        if (IS_DOUBLE_CODE(vCodeTable)) {
            for (int i = 0; i < pData->backspaceCount && !_syncKey.empty(); i++)
                _syncKey.pop_back();
            for (size_t i = 0; i < inserts.size(); i++)
                InsertKeyLength(inserts[i]);
        }
        if (pData->code == vRestoreAndStartNewSession)
            startNewSession();
        if (sendControl)
            SendKeyCode(_keycode);
        pData->backspaceCount = 0;
        return true;
    }

    static void selectInsteadOfBackspace(void) {
        if (pData->backspaceCount == 0)
            return;
        for (int i = 0; i < pData->backspaceCount; i++)
            SendShiftAndLeftArrow();
        pData->backspaceCount = 0;
    }

    void handleMacro() {
        if (isLauncherSearchApp(FRONT_APP)) {
            selectInsteadOfBackspace();
        } else if (shouldUseRecommendWorkaround(FRONT_APP)) {
            SendEmptyCharacter();
            pData->backspaceCount++;
        }
        
        //send backspace
        if (pData->backspaceCount > 0) {
            for (int i = 0; i < pData->backspaceCount; i++) {
                SendBackspace();
            }
        }
        //send real data
        if (!vSendKeyStepByStep) {
            SendNewCharString(true);
        } else {
            for (int i = 0; i < pData->macroData.size(); i++) {
                if (pData->macroData[i] & PURE_CHARACTER_MASK) {
                    SendPureCharacter(pData->macroData[i]);
                } else {
                    SendKeyCode(pData->macroData[i]);
                }
            }
        }
        SendKeyCode(_keycode | (_flag & kCGEventFlagMaskShift ? CAPS_MASK : 0));
    }

    // TODO: Research API to convert character into CGKeyCode more elegantly!
    int ConvertKeyStringToKeyCode(NSString *keyString, CGKeyCode fallback) {
        // Infomation about capitalization (shift/caps) is already included
        // in the original CGEvent, only find out which position on keyboard a key is pressed
        NSString *lowercasedKeyString = [keyString lowercaseString];
        if (!lowercasedKeyString) {
            return fallback;
        }
        
        NSNumber *keycode = [keyStringToKeyCodeMap objectForKey:lowercasedKeyString];

        if (keycode) {
            return [keycode intValue];
        }
        return fallback;
    }

    // If conversion fails, return fallbackKeyCode
    CGKeyCode ConvertEventToKeyboadLayoutCompatKeyCode(CGEventRef keyEvent, CGKeyCode fallbackKeyCode) {
        NSEvent *kbLayoutCompatEvent = [NSEvent eventWithCGEvent:keyEvent];
        NSString *kbLayoutCompatKeyString = kbLayoutCompatEvent.charactersIgnoringModifiers;
        return ConvertKeyStringToKeyCode(kbLayoutCompatKeyString,
                                         fallbackKeyCode);
    }

    /**
     * MAIN HOOK entry, very important function.
     * MAIN Callback.
     */
    static bool cachedOtherLanguageBlocksVietnamese(void) {
        if (!vOtherLanguage)
            return false;
        static CFAbsoluteTime checkedAt = 0;
        static bool blocked = false;
        CFAbsoluteTime now = CFAbsoluteTimeGetCurrent();
        if (now - checkedAt < 0.3)
            return blocked;
        checkedAt = now;
        blocked = false;
        TISInputSourceRef isource = TISCopyCurrentKeyboardInputSource();
        if (isource == NULL)
            return false;
        CFArrayRef languages = (CFArrayRef)TISGetInputSourceProperty(isource, kTISPropertyInputSourceLanguages);
        if (languages != NULL && CFArrayGetCount(languages) > 0) {
            CFStringRef langRef = (CFStringRef)CFArrayGetValueAtIndex(languages, 0);
            NSString* currentLanguage = (__bridge NSString*)langRef;
            if (currentLanguage != nil && ![currentLanguage hasPrefix:@"en"])
                blocked = true;
        }
        CFRelease(isource);
        return blocked;
    }

    CGEventRef OpenKeyCallback(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *refcon) {
        if (type == kCGEventTapDisabledByTimeout || type == kCGEventTapDisabledByUserInput) {
            OpenKeyEnsureEventTap();
            return NULL;
        }
        if (event == NULL)
            return NULL;

        //dont handle my event
        if (CGEventGetIntegerValueField(event, kCGEventSourceStateID) == CGEventSourceGetSourceStateID(myEventSource)) {
            return event;
        }
        refreshFrontAppFromEvent(event);
        
        _flag = CGEventGetFlags(event);
        _keycode = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
        
        if (type == kCGEventKeyDown && vPerformLayoutCompat) {
            // If conversion fail, use current keycode
           _keycode = ConvertEventToKeyboadLayoutCompatKeyCode(event, _keycode);
        }
        
        //switch language shortcut; convert hotkey
        if (type == kCGEventKeyDown) {
            if (GET_SWITCH_KEY(vSwitchKeyStatus) != _keycode && GET_SWITCH_KEY(convertToolHotKey) != _keycode) {
                _lastFlag = 0;
            } else {
                if (GET_SWITCH_KEY(vSwitchKeyStatus) == _keycode && checkHotKey(vSwitchKeyStatus, GET_SWITCH_KEY(vSwitchKeyStatus) != 0xFE)){
                    switchLanguage();
                    _lastFlag = 0;
                    _hasJustUsedHotKey = true;
                    return NULL;
                }
                if (GET_SWITCH_KEY(convertToolHotKey) == _keycode && checkHotKey(convertToolHotKey, GET_SWITCH_KEY(convertToolHotKey) != 0xFE)){
                    [appDelegate onQuickConvert];
                    _lastFlag = 0;
                    _hasJustUsedHotKey = true;
                    return NULL;
                }
            }
            _hasJustUsedHotKey = _lastFlag != 0;
        } else if (type == kCGEventFlagsChanged) {
            if (_lastFlag == 0 || _lastFlag < _flag) {
                _lastFlag = _flag;
            } else if (_lastFlag > _flag)  {
                //check switch
                if (checkHotKey(vSwitchKeyStatus, GET_SWITCH_KEY(vSwitchKeyStatus) != 0xFE)) {
                    _lastFlag = 0;
                    switchLanguage();
                    _hasJustUsedHotKey = true;
                    return NULL;
                }
                if (checkHotKey(convertToolHotKey, GET_SWITCH_KEY(convertToolHotKey) != 0xFE)) {
                    _lastFlag = 0;
                    [appDelegate onQuickConvert];
                    _hasJustUsedHotKey = true;
                    return NULL;
                }
                //check temporarily turn off spell checking
                if (vTempOffSpelling && !_hasJustUsedHotKey && _lastFlag & kCGEventFlagMaskControl) {
                    vTempOffSpellChecking();
                }
                if (vTempOffOpenKey && !_hasJustUsedHotKey && _lastFlag & kCGEventFlagMaskCommand) {
                    vTempOffEngine();
                }
                _lastFlag = 0;
                _hasJustUsedHotKey = false;
            }
        }

        // Also check correct event hooked
        if ((type != kCGEventKeyDown) && (type != kCGEventKeyUp) &&
            (type != kCGEventLeftMouseDown) && (type != kCGEventRightMouseDown) &&
            (type != kCGEventLeftMouseDragged) && (type != kCGEventRightMouseDragged))
            return event;
        
        _proxy = proxy;
        
        //If is in english mode
        if (vLanguage == 0) {
            if (vUseMacro && vUseMacroInEnglishMode && type == kCGEventKeyDown) {
                vEnglishMode((type == kCGEventKeyDown ? vKeyEventState::KeyDown : vKeyEventState::MouseDown),
                             _keycode,
                             (_flag & kCGEventFlagMaskShift) || (_flag & kCGEventFlagMaskAlphaShift),
                             OTHER_CONTROL_KEY);
                
                if (pData->code == vReplaceMaro) { //handle macro in english mode
                    handleMacro();
                    return NULL;
                }
            }
            return event;
        }
        
        //handle mouse
        if (type == kCGEventLeftMouseDown || type == kCGEventRightMouseDown || type == kCGEventLeftMouseDragged || type == kCGEventRightMouseDragged) {
            RequestNewSession();
            return event;
        }

        //if "turn off Vietnamese when in other language" mode on
        if (cachedOtherLanguageBlocksVietnamese())
            return event;
        
        //handle keyboard
        if (type == kCGEventKeyDown) {
            //send event signal to Engine
            vKeyHandleEvent(vKeyEvent::Keyboard,
                            vKeyEventState::KeyDown,
                            _keycode,
                            _flag & kCGEventFlagMaskShift ? 1 : (_flag & kCGEventFlagMaskAlphaShift ? 2 : 0),
                            OTHER_CONTROL_KEY);
            if (pData->code == vDoNothing) { //do nothing
                if (IS_DOUBLE_CODE(vCodeTable)) { //VNI
                    if (pData->extCode == 1) { //break key
                        _syncKey.clear();
                    } else if (pData->extCode == 2) { //delete key
                        if (_syncKey.size() > 0) {
                            if (_syncKey.back() > 1 && (vCodeTable == 2 || !containUnicodeCompoundApp(FRONT_APP))) {
                                //send one more backspace
                                CGEventTapPostEvent(_proxy, eventBackSpaceDown);
                                CGEventTapPostEvent(_proxy, eventBackSpaceUp);
                            }
                            _syncKey.pop_back();
                        }
                       
                    } else if (pData->extCode == 3) { //normal key
                        InsertKeyLength(1);
                    }
                }
                return event;
            } else if (pData->code == vWillProcess || pData->code == vRestore || pData->code == vRestoreAndStartNewSession) { //handle result signal
                
                if (isLauncherSearchApp(FRONT_APP)) {
                    if (commitLauncherComposition())
                        return NULL;
                    selectInsteadOfBackspace();
                } else if (shouldUseRecommendWorkaround(FRONT_APP) && pData->extCode != 4) {
                    //fix autocomplete
                    if (vFixChromiumBrowser && [_unicodeCompoundApp containsObject:FRONT_APP]) {
                        if (pData->backspaceCount > 0) {
                            SendShiftAndLeftArrow();
                            if (pData->backspaceCount == 1)
                                pData->backspaceCount--;
                        }
                    } else {
                        SendEmptyCharacter();
                        pData->backspaceCount++;
                    }
                }
                
                //send backspace
                if (pData->backspaceCount > 0 && pData->backspaceCount < MAX_BUFF) {
                    for (_i = 0; _i < pData->backspaceCount; _i++) {
                        SendBackspace();
                    }
                }
                
                //send new character
                if (!vSendKeyStepByStep) {
                    SendNewCharString();
                } else {
                    if (pData->newCharCount > 0 && pData->newCharCount <= MAX_BUFF) {
                        for (int i = pData->newCharCount - 1; i >= 0; i--) {
                            SendKeyCode(pData->charData[i]);
                        }
                    }
                    if (pData->code == vRestore || pData->code == vRestoreAndStartNewSession) {
                        SendKeyCode(_keycode | ((_flag & kCGEventFlagMaskAlphaShift) || (_flag & kCGEventFlagMaskShift) ? CAPS_MASK : 0));
                    }
                    if (pData->code == vRestoreAndStartNewSession) {
                        startNewSession();
                    }
                }
            } else if (pData->code == vReplaceMaro) { //MACRO
                handleMacro();
            }
            
            return NULL;
        }
        
        return event;
    }
}
