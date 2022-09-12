load("@rules_rust//rust:defs.bzl", "rust_library")

exports_files(glob(["*rs"]))

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
