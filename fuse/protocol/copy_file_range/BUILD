load("@rules_rust//rust:defs.bzl", "rust_test")
load("//fuse:protocol/common.bzl", "rust_fuse_protocol_module")

exports_files(glob(["*.rs"]))

rust_fuse_protocol_module(
    "copy_file_range",
    interop_test_os = ["linux"],
)
