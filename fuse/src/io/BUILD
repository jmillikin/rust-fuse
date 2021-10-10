load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_test")

filegroup(
    name = "srcs",
    srcs = glob(["*.rs"], exclude = ["*_test.rs"]),
    visibility = ["//fuse:__subpackages__"],
)

rust_test(
    name = "decode_test",
    srcs = ["decode_test.rs"] + [
        "//fuse:test_srcs",
    ],
    crate = "//fuse",
    crate_features = [
        "std",
    ],
    rustc_flags = ['--cfg=rust_fuse_test="decode_test"'],
)
