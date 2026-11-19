// Copyright 2023--present rgconf developers
//
// Nanobind extension in the rgsaddle style: the module links the rgconf C ABI
// and does not reimplement resolution. Build it with CMake when nanobind is
// installed. The PyO3 crate under python/ is the binding cargo tests build.

#include <nanobind/nanobind.h>
#include <nanobind/stl/string.h>

#include "rgconf.h"

namespace nb = nanobind;

NB_MODULE(_core, m) {
    m.def("abi_version", []() { return rgconf_abi_version(); });
    m.def("resolve_ini", [](const std::string &profile, const std::string &text, const std::string &path) {
        char *out = nullptr;
        int code = rgconf_resolve_ini(profile.c_str(), text.c_str(), path.c_str(), &out);
        if (code != 0) {
            std::string message = rgconf_last_error();
            throw nb::value_error(message.c_str());
        }
        std::string body = out == nullptr ? std::string() : std::string(out);
        rgconf_free(out);
        return body;
    });
}
