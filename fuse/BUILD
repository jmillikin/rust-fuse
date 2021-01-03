load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_test")

ALL_SRCS = glob(["src/**/*.rs"])

rust_library(
    name = "fuse",
    srcs = [src for src in ALL_SRCS if not src.endswith("_test.rs")],
    crate_features = [
        "std",
        "nightly_syscall_fuse_mount",
    ],
    visibility = ["//visibility:public"],
)

rust_test(
    name = "fuse_test",
    srcs = [src for src in ALL_SRCS if src.endswith("_test.rs")],
    crate = ":fuse",
    crate_features = [
        "std",
    ],
)
