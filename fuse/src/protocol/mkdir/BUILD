load("@io_bazel_rules_rust//rust:rust.bzl", "rust_test")

exports_files(glob(["*rs"]))

rust_test(
    name = "mkdir_test",
    srcs = ["mkdir_test.rs"] + [
        "//fuse:test_srcs",
    ],
    crate = "//fuse",
    crate_features = [
        "std",
    ],
)
