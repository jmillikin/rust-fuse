load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_cc",
    sha256 = "71d037168733f26d2a9648ad066ee8da4a34a13f51d24843a42efa6b65c2420f",
    strip_prefix = "rules_cc-b1c40e1de81913a3c40e5948f78719c28152486d",
    url = "https://github.com/bazelbuild/rules_cc/archive/b1c40e1de81913a3c40e5948f78719c28152486d.tar.gz",
)

http_archive(
    name = "io_bazel_rules_rust",
    patch_args = ["-p1"],
    sha256 = "8e1bae501e0df40e8feb2497ebab37c84930bf00b332f8f55315dfc08d85c30a",
    strip_prefix = "rules_rust-df18ddbece5b68f86e63414ea4b50d691923039a",
    urls = [
        # Master branch as of 2021-01-02
        "https://github.com/bazelbuild/rules_rust/archive/df18ddbece5b68f86e63414ea4b50d691923039a.tar.gz",
    ],
)

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2018",
    iso_date = "2020-12-30",
    sha256s = {
        "2020-12-30/rust-nightly-x86_64-apple-darwin": "2b5b885694d0d1a9bdd0473d9e2df1f2c6eac88986e3135e6573e1d71e7824dc",
        "2020-12-30/llvm-tools-nightly-x86_64-apple-darwin": "8aca7ddf73983bf2db4846721787547fed16c2ad4dc5c260f7f05f6b93cea8e7",
        "2020-12-30/rust-std-nightly-x86_64-apple-darwin": "17912a6a5aa56daeb0aed5fca8698bacc54950351d9f91989a524588e37e41ca",
        "2020-12-30/rust-std-nightly-armv7-unknown-linux-musleabihf": "c7176fe7fccd6ab71535ce1abf81ab71c8cfdffbaa0f51f71d1d13b7f4526f22",
        "2020-12-30/rust-std-nightly-x86_64-unknown-linux-musl": "3802d2c7271cdd3fc35921b0d9f999b9b34ac9d888b62085b976453a8b113700",
    },
    version = "nightly",
)

http_archive(
    name = "rust_diff",
    build_file_content = """
load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")
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
load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")
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
