#include <stdio.h>
#include <string.h>
#include <stdint.h>

/* Mirrors stringByReplacingComposition in OpenKey.mm (UTF-16 code units). */
static int composition_replace(const uint16_t *value, size_t value_len,
                               size_t caret, size_t selection_len,
                               size_t delete_count,
                               const uint16_t *replacement, size_t replacement_len,
                               uint16_t *out, size_t out_cap,
                               size_t *out_len, size_t *out_caret) {
    if (caret > value_len)
        caret = value_len;
    if (selection_len > value_len - caret)
        selection_len = value_len - caret;
    if (delete_count > caret)
        delete_count = caret;
    size_t start = caret - delete_count;
    size_t replace_len = delete_count + selection_len;
    size_t new_len = value_len - replace_len + replacement_len;
    if (new_len > out_cap)
        return 0;
    memcpy(out, value, start * sizeof(uint16_t));
    memcpy(out + start, replacement, replacement_len * sizeof(uint16_t));
    memcpy(out + start + replacement_len, value + start + replace_len,
           (value_len - (start + replace_len)) * sizeof(uint16_t));
    *out_len = new_len;
    *out_caret = start + replacement_len;
    return 1;
}

static int expect(const char *name, const uint16_t *got, size_t got_len, size_t got_caret,
                  const uint16_t *want, size_t want_len, size_t want_caret) {
    if (got_len != want_len || got_caret != want_caret ||
        memcmp(got, want, want_len * sizeof(uint16_t)) != 0) {
        fprintf(stderr, "FAIL %s\n", name);
        return 1;
    }
    printf("ok %s\n", name);
    return 0;
}

int main(void) {
    uint16_t out[32];
    size_t out_len = 0, out_caret = 0;
    int failed = 0;

    const uint16_t o[] = {0x006F};
    const uint16_t o_horn[] = {0x00F4};          /* ô */
    const uint16_t o_grave[] = {0x1ED3};         /* ồ */
    composition_replace(o, 1, 1, 0, 1, o_horn, 1, out, 32, &out_len, &out_caret);
    failed += expect("o -> o-horn", out, out_len, out_caret, o_horn, 1, 1);

    composition_replace(o_horn, 1, 1, 0, 1, o_grave, 1, out, 32, &out_len, &out_caret);
    failed += expect("o-horn -> o-grave", out, out_len, out_caret, o_grave, 1, 1);

    const uint16_t hello_o[] = {0x0068, 0x0065, 0x006C, 0x006C, 0x006F, 0x0020, 0x006F};
    const uint16_t hello_horn[] = {0x0068, 0x0065, 0x006C, 0x006C, 0x006F, 0x0020, 0x00F4};
    composition_replace(hello_o, 7, 7, 0, 1, o_horn, 1, out, 32, &out_len, &out_caret);
    failed += expect("hello o -> hello o-horn", out, out_len, out_caret, hello_horn, 7, 7);

    const uint16_t ab[] = {0x0061, 0x0062};
    const uint16_t horn_b[] = {0x00F4, 0x0062};
    composition_replace(ab, 2, 1, 0, 1, o_horn, 1, out, 32, &out_len, &out_caret);
    failed += expect("caret in the middle", out, out_len, out_caret, horn_b, 2, 1);

    const uint16_t empty[] = {0};
    composition_replace(empty, 0, 5, 3, 9, o_grave, 1, out, 32, &out_len, &out_caret);
    failed += expect("clamp past start", out, out_len, out_caret, o_grave, 1, 1);

    const uint16_t sel[] = {0x0061, 0x0058, 0x0059};
    const uint16_t sel_want[] = {0x00F4};
    composition_replace(sel, 3, 1, 2, 1, o_horn, 1, out, 32, &out_len, &out_caret);
    failed += expect("replace selection", out, out_len, out_caret, sel_want, 1, 1);

    if (failed) {
        fprintf(stderr, "%d failed\n", failed);
        return 1;
    }
    return 0;
}
