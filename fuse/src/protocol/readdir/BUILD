load("@io_bazel_rules_rust//rust:rust.bzl", "rust_test")

exports_files(glob(["*rs"]))

rust_test(
    name = "readdir_test",
    srcs = ["readdir_test.rs"] + [
        "//fuse:test_srcs",
    ],
    crate = "//fuse",
    crate_features = [
        "std",
    ],
)

rust_test(
    name = "readdir_interop_test",
    srcs = ["readdir_interop_test.rs"],
    crate_features = [
        "std",
    ],
    deps = [
        "//fuse",
        "//fuse/src/internal:interop_testutil",
        "@rust_libc//:libc",
    ],
)
