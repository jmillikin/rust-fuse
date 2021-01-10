load("@io_bazel_rules_rust//rust:rust.bzl", "rust_test")

exports_files(glob(["*rs"]))

rust_test(
    name = "statfs_test",
    srcs = ["statfs_test.rs"] + [
        "//fuse:test_srcs",
    ],
    crate = "//fuse",
    crate_features = [
        "std",
    ],
)

rust_test(
    name = "statfs_interop_test",
    srcs = ["statfs_interop_test.rs"],
    crate_features = [
        "std",
    ],
    deps = [
        "//fuse",
        "//fuse/src/internal:interop_testutil",
        "@rust_libc//:libc",
    ],
)
