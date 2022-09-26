load(
    "@rules_rust//rust:defs.bzl",
    "rust_clippy",
    "rust_doc",
    "rust_doc_test",
    "rust_library",
)

rust_library(
    name = "fuse-linux",
    srcs = glob(["*.rs"]),
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        "//fuse",
        "@rust_linux_errno//linux-errno",
        "@rust_linux_syscall//linux-syscall",
    ],
)

rust_clippy(
    name = "fuse-linux_clippy",
    deps = [":fuse-linux"],
)

rust_doc(
    name = "fuse-linux_doc",
    crate = ":fuse-linux",
)

rust_doc_test(
    name = "fuse-linux_doc_test",
    crate = ":fuse-linux",
)
