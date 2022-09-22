load("@rules_rust//rust:defs.bzl", "rust_test")

def rust_fuse_protocol_module(interop_test_os = None):
    files = native.glob(["*.rs"])
    name = native.package_name()[len("fuse/protocol/"):]

    if name + "_test.rs" in files:
        rust_test(
            name = name + "_test",
            srcs = [name + "_test.rs"] + [
                "//fuse:test_srcs",
            ],
            size = "small",
            timeout = "short",
            crate = "//fuse",
            crate_features = [
                "std",
                "unstable_" + name,
            ],
            rustc_flags = ['--cfg=rust_fuse_test="{}_test"'.format(name)],
        )

    if name + "_interop_test.rs" in files:
        test_name = name + "_interop_test"
        if test_name not in native.existing_rules():
            rust_test(
                name = test_name,
                srcs = [test_name + ".rs"],
                size = "medium",
                timeout = "short",
                crate_features = ["std"],
                deps = [
                    "//fuse",
                    "//fuse/internal:interop_testutil",
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
