/* GPL-3.0-only. C/ObjC bridge for the Rust engine; intentionally no C++ vector ABI. */
#ifndef NEOKEY_ENGINE_H
#define NEOKEY_ENGINE_H
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#ifdef __cplusplus
extern "C" {
#endif
/* One independent legacy engine per thread. Hook pointer lasts until thread exit;
 * vector pointers last until the next mutating call. Never free returned pointers.
 * Copy output before calling the engine again. Inputs must be valid buffers/UTF-8.
 * Platform event/key values are those in engine/platforms/mac.h. */
typedef struct {
    int64_t vLanguage;
    int64_t vInputType;
    int64_t vFreeMark;
    int64_t vCodeTable;
    int64_t vSwitchKeyStatus;
    int64_t vCheckSpelling;
    int64_t vUseModernOrthography;
    int64_t vQuickTelex;
    int64_t vRestoreIfWrongSpelling;
    int64_t vFixRecommendBrowser;
    int64_t vUseMacro;
    int64_t vUseMacroInEnglishMode;
    int64_t vAutoCapsMacro;
    int64_t vUseSmartSwitchKey;
    int64_t vUpperCaseFirstChar;
    int64_t vTempOffSpelling;
    int64_t vAllowConsonantZFWJ;
    int64_t vQuickStartConsonant;
    int64_t vQuickEndConsonant;
    int64_t vRememberCode;
    int64_t vOtherLanguage;
    int64_t vTempOffOpenKey;
} NeoKeySettings;
typedef struct {
    uint8_t code, backspaceCount, newCharCount, extCode;
    uint32_t charData[32];
    const uint32_t *macroKey; size_t macroKeyCount;
    const uint32_t *macroData; size_t macroDataCount;
} NeoKeyHookState;
typedef struct {
    bool dont_alert_when_completed, to_all_caps, to_all_non_caps;
    bool to_caps_first_letter, to_caps_each_word, remove_mark;
    uint8_t from_code, to_code; int32_t hot_key;
} NeoKeyConvertOptions;
const NeoKeyHookState *vKeyInit(void);
const NeoKeyHookState *vKeyGetHookState(void);
NeoKeySettings vKeyDefaultSettings(void);
NeoKeySettings vKeyGetSettings(void);
bool vKeySetSettings(const NeoKeySettings *settings);
void vKeyHandleEvent(uint8_t event, uint8_t state, uint16_t data, uint8_t caps, bool other_control);
void startNewSession(void);
uint32_t getCharacterCode(uint32_t data);
void vEnglishMode(uint8_t state, uint16_t data, bool caps, bool other_control);
void vTempOffSpellChecking(void);
void vSetCheckSpelling(void);
void vTempOffEngine(bool off);
bool initMacroMap(const uint8_t *data, size_t size);
bool addMacro(const char *key, const char *content);
bool deleteMacro(const char *key);
bool hasMacro(const char *key);
bool findMacro(const uint32_t *key, size_t count);
const uint8_t *getMacroSaveData(size_t *size);
void onTableCodeChange(void);
bool saveToFile(const char *path);
bool readFromFile(const char *path, bool append);
bool initSmartSwitchKey(const uint8_t *data, size_t size);
const uint8_t *getSmartSwitchKeySaveData(size_t *size);
int32_t getAppInputMethodStatus(const char *bundle, int32_t current);
void setAppInputMethodStatus(const char *bundle, int32_t language);
bool setConvertOptions(const NeoKeyConvertOptions *options);
const char *convertUtil(const char *source);
#ifdef __cplusplus
}
#endif
#endif
