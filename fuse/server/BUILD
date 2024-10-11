load("@rules_rust//rust:defs.bzl", "rust_test")

filegroup(
    name = "srcs",
    srcs = [
        "decode.rs",
    ],
    visibility = ["//fuse:__subpackages__"],
)

rust_test(
    name = "decode_test",
    size = "small",
    timeout = "short",
    srcs = ["decode_test.rs"],
    crate = "//fuse",
    rustc_flags = [
        '--cfg=rust_fuse_test="decode_test"',
        "--deny=warnings",
    ],
    deps = [
        "//fuse",
        "//fuse/internal/testing:fuse_testutil",
    ],
)
