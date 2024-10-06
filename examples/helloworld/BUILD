load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "helloworld",
    srcs = ["helloworld.rs"],
    edition = "2021",
    deps = [
        "//fuse",
        "//fuse-libc",
        "//fuse-std",
        "@rust_libc//:libc",
    ],
)
