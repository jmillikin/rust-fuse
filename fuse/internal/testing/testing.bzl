load("@rules_rust//rust:defs.bzl", "rust_test")

def operation_tests(name, interop_test_os = None):
    for filename in native.glob(["*.rs"]):
        if not filename.startswith(name + "_"):
            continue
        if filename.endswith("_interop_test.rs"):
            _operation_interop_test(filename, interop_test_os)
        elif filename.endswith("_test.rs"):
            _operation_test(filename)

def _operation_test(filename):
    target_name = filename[:-len(".rs")]
    if target_name in native.existing_rules():
        return

    rust_test(
        name = target_name,
        srcs = [filename],
        size = "small",
        timeout = "short",
        deps = [
            "//fuse",
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

def _operation_interop_test(filename, interop_test_os):
    target_name = filename[:-len(".rs")]
    if target_name in native.existing_rules():
        return

    rust_test(
        name = target_name,
        srcs = [filename],
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
