load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "helloworld",
    srcs = ["helloworld.rs"],
    edition = "2018",
    deps = [
        "//fuse",
        "//fuse-libc",
        "@rust_libc//:libc",
    ],
)
