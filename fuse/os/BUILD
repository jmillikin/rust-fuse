load("@rules_rust//rust:defs.bzl", "rust_test")

filegroup(
    name = "srcs",
    srcs = [
        "freebsd.rs",
        "linux.rs",
        "node_name.rs",
        "xattr.rs",
    ],
    visibility = ["//:__subpackages__"],
)

rust_test(
    name = "linux_test",
    srcs = [
        "linux.rs",
        "linux_test.rs",
    ],
    crate = "//fuse",
    edition = "2021",
    rustc_flags = ["--deny=warnings"],
    target_compatible_with = [
        "@platforms//os:linux",
    ],
)
