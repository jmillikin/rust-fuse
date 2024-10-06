load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

package(
    default_testonly = True,
    default_visibility = ["//fuse:__subpackages__"],
)

rust_library(
    name = "fuse_testutil",
    srcs = ["fuse_testutil.rs"],
    edition = "2021",
    deps = ["//fuse"],
)

rust_library(
    name = "interop_testutil",
    srcs = ["interop_testutil.rs"],
    edition = "2021",
    deps = [
        ":fuse_testutil",
        "//fuse",
        "@rust_diff//:diff",
        "@rust_libc//:libc",
    ] + select({
        "@platforms//os:freebsd": [
            "//fuse-libc",
            "@rust_freebsd_errno//freebsd-errno",
        ],
        "@platforms//os:linux": [
            "//fuse-linux",
            "@rust_linux_errno//linux-errno",
        ],
        "//conditions:default": [],
    }),
)

rust_binary(
    name = "benchmark_io",
    testonly = True,
    srcs = ["benchmark_io.rs"],
    args = ["--bench"],
    edition = "2021",
    rustc_flags = [
        "-Copt-level=3",
    ],
    tags = ["manual"],
    deps = [
        "//fuse",
        "//fuse/internal/testing:fuse_testutil",
        "@rust_criterion//:criterion",
    ],
)
