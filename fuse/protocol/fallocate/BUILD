load("//fuse:protocol/common.bzl", "rust_fuse_protocol_module")

exports_files(glob(["*.rs"]))

rust_fuse_protocol_module(
    "fallocate",
    interop_test_os = ["linux"],
)
