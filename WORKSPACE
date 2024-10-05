load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "696b01deea96a5e549f1b5ae18589e1bbd5a1d71a36a243b5cf76a9433487cf2",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_rust/releases/download/0.11.0/rules_rust-v0.11.0.tar.gz",
        "https://github.com/bazelbuild/rules_rust/releases/download/0.11.0/rules_rust-v0.11.0.tar.gz",
    ],
)

http_archive(
    name = "rust_freebsd_errno",
    sha256 = "04afa4cca65b47d1093b0fd60868f3ad50a8b08194cf6fd2a497e2859333eef1",
    strip_prefix = "freebsd-errno-1.0.0",
    urls = ["https://github.com/jmillikin/rust-freebsd-errno/releases/download/v1.0.0/freebsd-errno-1.0.0.tar.xz"],
)

http_archive(
    name = "rust_linux_errno",
    sha256 = "009d58c93c806f178004a4cd30af211860bc44f8ce7d02eb4f544821add7ca99",
    strip_prefix = "linux-errno-1.0.1",
    urls = ["https://github.com/jmillikin/rust-linux-errno/releases/download/v1.0.1/linux-errno-1.0.1.tar.xz"],
)

http_archive(
    name = "rust_linux_syscall",
    sha256 = "6e26b9e20d8795100b9035c698f4977ad153bed1eafe9f9dcb54f14fbd2b120a",
    strip_prefix = "linux-syscall-1.0.0",
    urls = ["https://github.com/jmillikin/rust-linux-syscall/releases/download/v1.0.0/linux-syscall-1.0.0.tar.xz"],
)

http_archive(
    name = "rust_posix_errno",
    sha256 = "0c86c849ff673372fe6415d4004a233565b57b2884ea49d3b725dd1296cc2529",
    strip_prefix = "posix-errno-1.0.1",
    urls = ["https://github.com/jmillikin/rust-posix-errno/releases/download/v1.0.1/posix-errno-1.0.1.tar.xz"],
)

http_archive(
    name = "rust_diff",
    build_file_content = """
load("@rules_rust//rust:defs.bzl", "rust_library")
rust_library(
    name = "diff",
    srcs = glob(["src/*.rs"]),
    edition = "2015",
    visibility = ["//visibility:public"],
)
""",
    sha256 = "0e25ea47919b1560c4e3b7fe0aaab9becf5b84a10325ddf7db0f0ba5e1026499",
    strip_prefix = "diff-0.1.12",
    type = "tar.gz",
    url = "https://crates.io/api/v1/crates/diff/0.1.12/download",
)

http_archive(
    name = "rust_libc",
    build_file_content = """
load("@rules_rust//rust:defs.bzl", "rust_library")
rust_library(
    name = "libc",
    srcs = glob(["**/*.rs"]),
    crate_features = ["std"],
    edition = "2015",
    visibility = ["//visibility:public"],
    rustc_flags = select({
        "@platforms//os:freebsd": ["--cfg=freebsd12"],
        "//conditions:default": [],
    }),
)
""",
    sha256 = "c0f80d65747a3e43d1596c7c5492d95d5edddaabd45a7fcdb02b95f644164966",
    strip_prefix = "libc-0.2.133",
    type = "tar.gz",
    url = "https://crates.io/api/v1/crates/libc/0.2.133/download",
)

load(
    "@rules_rust//rust:repositories.bzl",
    "rules_rust_dependencies",
    "rust_register_toolchains",
)

rules_rust_dependencies()

rust_register_toolchains(
    edition = "2021",
    version = "1.64.0",
)
