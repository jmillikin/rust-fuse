load(
    "@rules_rust//rust:defs.bzl",
    "rust_clippy",
    "rust_doc",
    "rust_doc_test",
    "rust_library",
)

rust_library(
    name = "fuse-std",
    srcs = ["fuse-std.rs"],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = ["//fuse"],
)

rust_clippy(
    name = "fuse-std_clippy",
    deps = [":fuse-std"],
)

rust_doc(
    name = "fuse-std_doc",
    crate = ":fuse_std",
)

rust_doc_test(
    name = "fuse-std_doc_test",
    crate = ":fuse-std",
)
