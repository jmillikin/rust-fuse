load("@io_bazel_rules_rust//rust:rust.bzl", "rust_test")

exports_files(glob(["*rs"]))

rust_test(
    name = "listxattr_test",
    srcs = ["listxattr_test.rs"] + [
        "//fuse:test_srcs",
    ],
    crate = "//fuse",
    crate_features = [
        "std",
    ],
)

rust_test(
    name = "listxattr_interop_test",
    srcs = ["listxattr_interop_test.rs"],
    crate_features = [
        "std",
    ],
    deps = [
        "//fuse",
        "//fuse/src/internal:interop_testutil",
        "@rust_libc//:libc",
    ],
)
