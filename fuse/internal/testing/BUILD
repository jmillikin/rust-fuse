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
    tags = ["manual"],
    deps = [
        ":fuse_testutil",
        "//fuse",
        "@com_github_rust-lang_libc//:libc",
        "@com_github_utkarshkukreti_diff.rs//:diff",
    ] + select({
        "@platforms//os:freebsd": [
            "//fuse-libc",
            "@com_github_jmillikin_rust-freebsd-errno//freebsd-errno",
        ],
        "@platforms//os:linux": [
            "//fuse-linux",
            "@com_github_jmillikin_rust-linux-errno//linux-errno",
        ],
        "//conditions:default": [],
    }),
)
