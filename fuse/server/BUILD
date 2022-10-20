load("@rules_rust//rust:defs.bzl", "rust_test")

filegroup(
    name = "srcs",
    srcs = [
        "cuse_rpc.rs",
        "decode.rs",
        "encode.rs",
        "fuse_rpc.rs",
        "io.rs",
        "prelude.rs",
    ],
    visibility = ["//fuse:__subpackages__"],
)

rust_test(
    name = "decode_test",
    size = "small",
    timeout = "short",
    srcs = ["decode_test.rs"],
    crate = "//fuse",
    crate_features = [
        "alloc",
        "std",
    ],
    rustc_flags = ['--cfg=rust_fuse_test="decode_test"'],
    deps = [
        "//fuse",
        "//fuse/internal/testing:fuse_testutil",
    ],
)
