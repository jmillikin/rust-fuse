load("@rules_rust//rust:defs.bzl", "rust_test")

filegroup(
    name = "srcs",
    srcs = [
        "freebsd.rs",
        "linux.rs",
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
    edition = "2018",
    target_compatible_with = [
        "@platforms//os:linux",
    ],
)
