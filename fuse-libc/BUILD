load(
    "@rules_rust//rust:defs.bzl",
    "rust_clippy",
    "rust_doc",
    "rust_doc_test",
    "rust_library",
)

rust_library(
    name = "fuse-libc",
    srcs = glob([
        "fuse-libc.rs",
        "io/*.rs",
        "os/*.rs",
    ]),
    crate_features = ["std"],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        "//fuse",
        "@rust_libc//:libc",
    ],
)

rust_clippy(
    name = "fuse-libc_clippy",
    deps = [":fuse-libc"],
)

rust_doc(
    name = "fuse-libc_doc",
    crate = ":fuse-libc",
)

rust_doc_test(
    name = "fuse-libc_doc_test",
    crate = ":fuse-libc",
)
