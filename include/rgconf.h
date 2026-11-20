/* Copyright 2023--present rgconf developers */
#ifndef RGCONF_H
#define RGCONF_H

#include <stdint.h>

#define RGCONF_ABI_VERSION_MAJOR 0
#define RGCONF_ABI_VERSION_MINOR 1
#define RGCONF_ABI_VERSION_PATCH 0
#define RGCONF_ABI_VERSION \
    ((RGCONF_ABI_VERSION_MAJOR << 16) | (RGCONF_ABI_VERSION_MINOR << 8) | RGCONF_ABI_VERSION_PATCH)

#ifdef __cplusplus
extern "C" {
#endif

/* Numeric ABI. The macros above are the compile time contract. */
uint32_t rgconf_abi_version(void);

/* Pointer from the last failing call on this thread. Empty when the last
 * call succeeded. Invalidated by the next rgconf call on this thread. */
const char *rgconf_last_error(void);

/* Release a buffer returned by rgconf_resolve_ini or rgconf_apply_edit. */
void rgconf_free(char *ptr);

/* Resolve an INI document. On success *out_text is a heap buffer of
 * `key=value` lines and the return value is 0. */
int rgconf_resolve_ini(const char *profile, const char *text, const char *path, char **out_text);

/* Apply one runtime edit to a fresh profile resolution.
 * checkpoint is 0 for a live call and 1 for a checkpoint restart. */
int rgconf_apply_edit(const char *profile, const char *key, const char *value, int checkpoint, char **out_text);

#ifdef __cplusplus
}
#endif

#endif /* RGCONF_H */
