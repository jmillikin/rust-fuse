load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_test")

exports_files(glob(["*rs"]))

rust_test(
    name = "fuse_io_test",
    srcs = ["fuse_io_test.rs"] + [
        "//fuse:test_srcs",
    ],
    crate = "//fuse",
    crate_features = [
        "std",
    ],
    rustc_flags = ['--cfg=rust_fuse_test="fuse_io_test"'],
)

rust_library(
    name = "interop_testutil",
    srcs = ["interop_testutil.rs"],
    visibility = ["//fuse:__subpackages__"],
    deps = [
        "//fuse",
        "@rust_diff//:diff",
        "@rust_libc//:libc",
    ],
)
