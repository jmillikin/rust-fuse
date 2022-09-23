load("@rules_rust//rust:defs.bzl", "rust_doc", "rust_doc_test", "rust_library")

ALL_SRCS = glob(["**/*.rs"])

filegroup(
    name = "test_srcs",
    srcs = ["//fuse/internal:testutil.rs"],
    visibility = ["//fuse:__subpackages__"],
)

rust_library(
    name = "fuse",
    srcs = [src for src in ALL_SRCS if not src.endswith("_test.rs")] + [
        "//fuse/internal:fuse_kernel_util.rs",
        "//fuse/internal:fuse_kernel.rs",
        "//fuse/internal:mod.rs",
        "//fuse/operations:srcs",
        "//fuse/os:srcs",
        "//fuse/server/io:srcs",
    ],
    crate_features = ["std"],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = select({
        "@platforms//os:freebsd": [
            "@rust_freebsd_errno//freebsd-errno",
        ],
        "@platforms//os:linux": [
            "@rust_linux_errno//linux-errno",
        ],
        "//conditions:default": [],
    }),
)

rust_doc(
    name = "fuse_doc",
    crate = ":fuse",
)

rust_doc_test(
    name = "fuse_doc_test",
    crate = ":fuse",
)
