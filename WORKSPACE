load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "0cc7e6b39e492710b819e00d48f2210ae626b717a3ab96e048c43ab57e61d204",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.10.0/rules_rust-v0.10.0.tar.gz"],
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
    sha256 = "4d58d1b70b004888f764dfbf6a26a3b0342a1632d33968e4a179d8011c760614",
    strip_prefix = "libc-0.2.80",
    type = "tar.gz",
    url = "https://crates.io/api/v1/crates/libc/0.2.80/download",
)

load(
    "@rules_rust//rust:repositories.bzl",
    "rules_rust_dependencies",
    "rust_register_toolchains",
)

rules_rust_dependencies()

rust_register_toolchains(
    edition = "2018",
    version = "1.63.0",
)
