load("@io_bazel_rules_rust//rust:rust.bzl", "rust_test")

exports_files(glob(["*rs"]))

rust_test(
    name = "fuse_init_test",
    srcs = ["fuse_init_test.rs"] + [
        "//fuse:test_srcs",
    ],
    crate = "//fuse",
    crate_features = [
        "std",
    ],
)
