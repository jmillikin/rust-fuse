load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

filegroup(
    name = "srcs",
    srcs = glob(
        ["*.rs"],
        exclude = ["*_test.rs"],
    ),
    visibility = ["//fuse:__subpackages__"],
)

rust_test(
    name = "decode_test",
    size = "small",
    timeout = "short",
    srcs = ["decode_test.rs"],
    crate = "//fuse",
    crate_features = ["std"],
    rustc_flags = ['--cfg=rust_fuse_test="decode_test"'],
    deps = [
        "//fuse/internal/testing:fuse_testutil",
    ],
)
