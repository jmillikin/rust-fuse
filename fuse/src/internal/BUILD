load("@rules_rust//rust:defs.bzl", "rust_library")

exports_files(glob(["*rs"]))

rust_library(
    name = "interop_testutil",
    srcs = ["interop_testutil.rs"],
    edition = "2018",
    visibility = ["//fuse:__subpackages__"],
    deps = [
        "//fuse",
        "@rust_diff//:diff",
        "@rust_libc//:libc",
    ] + select({
        "@platforms//os:freebsd": [
            "@rust_freebsd_errno//freebsd-errno",
            "//fuse-libc",
        ],
        "@platforms//os:linux": [
            "@rust_linux_errno//linux-errno",
            "//fuse-linux",
        ],
        "//conditions:default": [],
    }),
)
