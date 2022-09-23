load("@rules_rust//rust:defs.bzl", "rust_doc", "rust_doc_test", "rust_library")

filegroup(
    name = "test_srcs",
    srcs = ["//fuse/internal:testutil.rs"],
    visibility = ["//fuse:__subpackages__"],
)

rust_library(
    name = "fuse",
    srcs = glob(["**/*.rs"]) + [
        "//fuse/client:srcs",
        "//fuse/internal:srcs",
        "//fuse/operations:srcs",
        "//fuse/os:srcs",
        "//fuse/server:srcs",
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
