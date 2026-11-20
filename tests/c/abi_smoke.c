/* Copyright 2023--present rgconf developers */
#include "rgconf.h"

#include <stdio.h>
#include <string.h>

int main(void) {
    uint32_t packed = (RGCONF_ABI_VERSION_MAJOR << 16) | (RGCONF_ABI_VERSION_MINOR << 8) | RGCONF_ABI_VERSION_PATCH;
    if (rgconf_abi_version() != packed || rgconf_abi_version() != RGCONF_ABI_VERSION) {
        fprintf(stderr, "abi mismatch\n");
        return 1;
    }
    const char *ini = "[Optimizer]\nconverged_force = 0.03\n";
    char *out = NULL;
    if (rgconf_resolve_ini("eonLegacyIni", ini, "client.ini", &out) != 0) {
        fprintf(stderr, "%s\n", rgconf_last_error());
        return 1;
    }
    if (strstr(out, "optimizer.convergence.forceToleranceEvPerAngstrom=0.03") == NULL) {
        fprintf(stderr, "missing force line\n%s\n", out);
        rgconf_free(out);
        return 1;
    }
    rgconf_free(out);

    out = (char *)1;
    if (rgconf_resolve_ini(NULL, ini, "client.ini", &out) == 0 || out != NULL ||
        strstr(rgconf_last_error(), "profile is null") == NULL) {
        fprintf(stderr, "null profile was not rejected safely\n");
        return 1;
    }
    out = (char *)1;
    if (rgconf_resolve_ini("eonLegacyIni", NULL, "client.ini", &out) == 0 || out != NULL ||
        strstr(rgconf_last_error(), "text is null") == NULL) {
        fprintf(stderr, "null text was not rejected safely\n");
        return 1;
    }
    out = (char *)1;
    if (rgconf_resolve_ini("eonLegacyIni", ini, NULL, &out) == 0 || out != NULL ||
        strstr(rgconf_last_error(), "path is null") == NULL) {
        fprintf(stderr, "null path was not rejected safely\n");
        return 1;
    }
    if (rgconf_resolve_ini("eonLegacyIni", ini, "client.ini", NULL) == 0 ||
        strstr(rgconf_last_error(), "out_text is null") == NULL) {
        fprintf(stderr, "null output was not rejected safely\n");
        return 1;
    }
    rgconf_free(NULL);
    return 0;
}
