load("@rules_rust//rust:defs.bzl", "rust_library")

package(
    default_testonly = True,
    default_visibility = ["//fuse:__subpackages__"],
)

rust_library(
    name = "fuse_testutil",
    srcs = ["fuse_testutil.rs"],
    edition = "2018",
    deps = [
        "//fuse/internal:fuse_kernel",
    ],
)

rust_library(
    name = "interop_testutil",
    srcs = ["interop_testutil.rs"],
    edition = "2018",
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
