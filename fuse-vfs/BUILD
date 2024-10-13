load(
    "@rules_rust//rust:defs.bzl",
    "rust_clippy",
    "rust_doc",
    "rust_doc_test",
    "rust_library",
)

rust_library(
    name = "fuse-vfs",
    srcs = ["fuse-vfs.rs"],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = ["//fuse"],
)

rust_clippy(
    name = "fuse-vfs_clippy",
    deps = [":fuse-vfs"],
)

rust_doc(
    name = "fuse-vfs_doc",
    crate = ":fuse-vfs",
)

rust_doc_test(
    name = "fuse-vfs_doc_test",
    size = "small",
    crate = ":fuse-vfs",
)
