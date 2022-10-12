load("@rules_rust//rust:defs.bzl", "rust_library")

filegroup(
    name = "srcs",
    srcs = [
        "bitflags.rs",
        "compat.rs",
        "debug.rs",
        "dirent.rs",
        "fuse_kernel.rs",
        "fuse_kernel_util.rs",
        "timestamp.rs",
    ],
    visibility = ["//fuse:__subpackages__"],
)

rust_library(
    name = "fuse_kernel",
    srcs = [
        "fuse_kernel.rs",
        "fuse_kernel_lib.rs",
        ":fuse_kernel_util_pub.rs",
    ],
    crate_root = "fuse_kernel_lib.rs",
    edition = "2018",
    visibility = ["//fuse:__subpackages__"],
)

genrule(
    name = "fuse_kernel_util_pub",
    srcs = ["fuse_kernel_util.rs"],
    outs = ["fuse_kernel_util_pub.rs"],
    cmd = "sed 's/pub(crate)/pub/g' $< > $@",
)
