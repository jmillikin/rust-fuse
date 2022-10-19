load(
    "@rules_rust//rust:defs.bzl",
    "rust_clippy",
    "rust_doc",
    "rust_doc_test",
    "rust_library",
)

rust_library(
    name = "fuse",
    srcs = glob(["*.rs"]) + [
        "//fuse/client:srcs",
        "//fuse/internal:srcs",
        "//fuse/operations:srcs",
        "//fuse/os:srcs",
        "//fuse/server:srcs",
        "//fuse/server_async:srcs",
    ],
    crate_features = [
        "alloc",
        "std",
    ],
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

rust_clippy(
    name = "fuse_clippy",
    deps = [":fuse"],
)

rust_doc(
    name = "fuse_doc",
    crate = ":fuse",
)

rust_doc_test(
    name = "fuse_doc_test",
    crate = ":fuse",
)
