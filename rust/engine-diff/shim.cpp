#include "Engine.h"
#include <cstdint>
#include <cstring>
int vLanguage=0;
int vInputType=0;
int vFreeMark=0;
int vCodeTable=0;
int vSwitchKeyStatus=0;
int vCheckSpelling=0;
int vUseModernOrthography=0;
int vQuickTelex=0;
int vRestoreIfWrongSpelling=0;
int vFixRecommendBrowser=0;
int vUseMacro=0;
int vUseMacroInEnglishMode=0;
int vAutoCapsMacro=0;
int vUseSmartSwitchKey=0;
int vUpperCaseFirstChar=0;
int vTempOffSpelling=0;
int vAllowConsonantZFWJ=0;
int vQuickStartConsonant=0;
int vQuickEndConsonant=0;
int vRememberCode=0;
int vOtherLanguage=0;
int vTempOffOpenKey=0;
#include "Engine.cpp"
extern "C" {
void ref_settings(const int64_t* v) {
vLanguage=static_cast<int>(v[0]);
vInputType=static_cast<int>(v[1]);
vFreeMark=static_cast<int>(v[2]);
vCodeTable=static_cast<int>(v[3]);
vSwitchKeyStatus=static_cast<int>(v[4]);
vCheckSpelling=static_cast<int>(v[5]);
vUseModernOrthography=static_cast<int>(v[6]);
vQuickTelex=static_cast<int>(v[7]);
vRestoreIfWrongSpelling=static_cast<int>(v[8]);
vFixRecommendBrowser=static_cast<int>(v[9]);
vUseMacro=static_cast<int>(v[10]);
vUseMacroInEnglishMode=static_cast<int>(v[11]);
vAutoCapsMacro=static_cast<int>(v[12]);
vUseSmartSwitchKey=static_cast<int>(v[13]);
vUpperCaseFirstChar=static_cast<int>(v[14]);
vTempOffSpelling=static_cast<int>(v[15]);
vAllowConsonantZFWJ=static_cast<int>(v[16]);
vQuickStartConsonant=static_cast<int>(v[17]);
vQuickEndConsonant=static_cast<int>(v[18]);
vRememberCode=static_cast<int>(v[19]);
vOtherLanguage=static_cast<int>(v[20]);
vTempOffOpenKey=static_cast<int>(v[21]);
onTableCodeChange();vSetCheckSpelling();}
void ref_reset() {
_hasHandleQuickConsonant=0;
_hasHandledMacro=0;
_index=0;
_isCaps=0;
_isCharKeyCode=0;
_spaceCount=0;
_spellingEndIndex=0;
_spellingFlag=0;
_spellingOK=0;
_spellingVowelOK=0;
_stateIndex=0;
_upperCaseStatus=0;
_useSpellCheckingBefore=0;
_willTempOffEngine=0;
capsElem=0;
i=0;
ii=0;
iii=0;
isChanged=0;
isCheckedGrammar=0;
isCorect=0;
isRestoredW=0;
j=0;
k=0;
key=0;
keyForAEO=0;
kk=0;
l=0;
markElem=0;
tempDisableKey=0;
vowelCount=0;
vowelEndIndex=0;
vowelStartIndex=0;
vowelWillSetMark=0;
HookState = vKeyHookState{};
std::memset(TypingWord,0,sizeof(TypingWord));std::memset(KeyStates,0,sizeof(KeyStates));
_longWordHelper.clear();_typingStates.clear();_typingStatesData.clear();_specialChar.clear();
initMacroMap(nullptr,0);vKeyInit();startNewSession();}

uint8_t ref_event(uint8_t event, uint8_t state, uint16_t data, uint8_t caps, bool ctrl) {
    vKeyHandleEvent(static_cast<vKeyEvent>(event), static_cast<vKeyEventState>(state), data, caps, ctrl);
    // Keep all hook writes observable under LTO without copying the result.
    asm volatile("" : : "r"(&HookState) : "memory");
    return HookState.code;
}
void ref_english(uint8_t state,uint16_t data,bool caps,bool ctrl) {vEnglishMode(static_cast<vKeyEventState>(state),data,caps,ctrl);}
void ref_session() {startNewSession();}
void ref_temp_spell() {vTempOffSpellChecking();}
void ref_temp_engine(bool off) {vTempOffEngine(off);}
uint16_t ref_key_character(uint32_t data) {return keyCodeToCharacter(data);}
uint32_t ref_character(uint32_t data) {return getCharacterCode(data);}
void ref_hook(uint8_t* head,uint32_t* chars) {
 head[0]=HookState.code;head[1]=HookState.backspaceCount;head[2]=HookState.newCharCount;head[3]=HookState.extCode;
 std::memcpy(chars,HookState.charData,sizeof(HookState.charData));
}
const uint32_t* ref_macro(bool key,size_t* n) {
 auto& v=key?HookState.macroKey:HookState.macroData;*n=v.size();return v.data();
}
bool ref_add_macro(const char* key,const char* value) {return addMacro(key,value);}
bool ref_delete_macro(const char* key) {return deleteMacro(key);}
void ref_init_macro(const uint8_t* bytes,int n) {initMacroMap(bytes,n);}
const uint8_t* ref_save_macro(size_t* n) {static std::vector<Byte> v;v.clear();getMacroSaveData(v);*n=v.size();return v.data();}
void ref_init_smart(const uint8_t* bytes,int n) {initSmartSwitchKey(bytes,n);}
int ref_get_smart(const char* key,int current) {return getAppInputMethodStatus(key,current);}
void ref_set_smart(const char* key,int v) {setAppInputMethodStatus(key,v);}
const uint8_t* ref_save_smart(size_t* n) {static std::vector<Byte> v;getSmartSwitchKeySaveData(v);*n=v.size();return v.data();}
const char* ref_convert(const char* text,uint8_t from,uint8_t to,uint32_t flags) {
 convertToolFromCode=from;convertToolToCode=to;convertToolToAllCaps=flags&1;convertToolToAllNonCaps=flags&2;
 convertToolToCapsFirstLetter=flags&4;convertToolToCapsEachWord=flags&8;convertToolRemoveMark=flags&16;
 static std::string result;try {result=convertUtil(text);} catch (const std::range_error&) {return nullptr;} return result.c_str();
}
}
