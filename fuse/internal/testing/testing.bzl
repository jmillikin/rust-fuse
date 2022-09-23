load("@rules_rust//rust:defs.bzl", "rust_test")

def operation_tests(name, interop_test_os = None):
    files = native.glob(["*.rs"])

    if name + "_test.rs" in files:
        rust_test(
            name = name + "_test",
            srcs = [name + "_test.rs"],
            size = "small",
            timeout = "short",
            deps = [
                "//fuse",
                "//fuse/internal:fuse_kernel",
                "//fuse/internal/testing:fuse_testutil",
            ] + select({
                "@platforms//os:freebsd": [
                    "@rust_freebsd_errno//freebsd-errno",
                ],
                "@platforms//os:linux": [
                    "@rust_linux_errno//linux-errno",
                ],
                "//conditions:default": [],
            }),
        )

    if name + "_interop_test.rs" in files:
        test_name = name + "_interop_test"
        if test_name not in native.existing_rules():
            rust_test(
                name = test_name,
                srcs = [test_name + ".rs"],
                size = "medium",
                timeout = "short",
                deps = [
                    "//fuse",
                    "//fuse/internal/testing:interop_testutil",
                    "@rust_libc//:libc",
                ] + select({
                    "@platforms//os:linux": [
                        "@rust_linux_errno//linux-errno",
                        "@rust_linux_syscall//linux-syscall",
                    ],
                    "//conditions:default": [],
                }),
                tags = ["manual"],
            )
