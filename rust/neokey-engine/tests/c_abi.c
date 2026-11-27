#include "neokey_engine.h"
#include <assert.h>
#include <string.h>
int main(void) {
    NeoKeySettings settings = vKeyDefaultSettings();
    settings.vCheckSpelling = 0;
    assert(vKeySetSettings(&settings));
    const NeoKeyHookState *hook = vKeyInit();
    vKeyHandleEvent(0, 0, 0, 0, false); /* a */
    vKeyHandleEvent(0, 0, 1, 0, false); /* s */
    assert(hook == vKeyGetHookState());
    assert(hook->code == 1 && hook->backspaceCount == 1 && hook->newCharCount == 1);
    assert(hook->charData[0] == (0xE1u | 0x2000000u));
    assert(getCharacterCode(0x80000) == (0xE1u | 0x2000000u));
    startNewSession();
    assert(initMacroMap(NULL, 0));
    assert(addMacro("btw", "hello"));
    assert(hasMacro("btw"));
    const uint32_t key[] = {11, 17, 13};
    assert(findMacro(key, 3));
    assert(hook->macroDataCount == 5);
    size_t size;
    assert(getMacroSaveData(&size) != NULL && size > 2);
    assert(deleteMacro("btw"));
    assert(initSmartSwitchKey(NULL, 0));
    assert(getAppInputMethodStatus("org.editor", 1) == -1);
    setAppInputMethodStatus("org.editor", 0);
    assert(getAppInputMethodStatus("org.editor", 1) == 0);
    assert(getSmartSwitchKeySaveData(&size) != NULL && size > 2);
    NeoKeyConvertOptions options = {0};
    options.to_all_caps = true;
    assert(setConvertOptions(&options));
    assert(strcmp(convertUtil("abc"), "ABC") == 0);
    assert(!addMacro(NULL, NULL));
    assert(!initMacroMap(NULL, 3));
    assert(!setConvertOptions(NULL));
    settings.vCheckSpelling = 1;
    assert(vKeySetSettings(&settings));
    vTempOffSpellChecking();
    assert(vKeyGetSettings().vCheckSpelling == 0);
    onTableCodeChange();
    vKeyHandleEvent(0, 0, 49, 0, false);
    assert(vKeyGetSettings().vCheckSpelling == 1);
    vTempOffEngine(true);vTempOffEngine(false);vTempOffSpellChecking();vSetCheckSpelling();
    return 0;
}
