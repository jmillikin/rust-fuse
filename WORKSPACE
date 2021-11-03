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
    patches = [
        "//build:rules_rust-triple-mappings.patch",
        "//build:rules_rust-no-skylib.patch",
        "//build:rules_rust-crate-name-no-slashes.patch",
    ],
    sha256 = "8e1bae501e0df40e8feb2497ebab37c84930bf00b332f8f55315dfc08d85c30a",
    strip_prefix = "rules_rust-df18ddbece5b68f86e63414ea4b50d691923039a",
    urls = [
        # Master branch as of 2021-01-02
        "https://github.com/bazelbuild/rules_rust/archive/df18ddbece5b68f86e63414ea4b50d691923039a.tar.gz",
    ],
)

local_repository(
    name = "rust_fuse_cc_toolchains",
    path = "build/cc_toolchains",
)

load("@rust_fuse_cc_toolchains//:cc_toolchains.bzl", "cc_toolchains")

cc_toolchains()

load("//build:rust_toolchains.bzl", "rust_toolchains")

rust_toolchains()

load(
    "//build/testutil:testutil.bzl",
    "busybox_multiarch",
    "freebsd_repository",
    "qemu_repository",
)

busybox_multiarch(name = "busybox_multiarch")

freebsd_repository(
    name = "freebsd_amd64_v12.2",
    platform = "amd64/amd64",
    version = "12.2",
)

freebsd_repository(
    name = "freebsd_amd64_v13.0",
    platform = "amd64/amd64",
    version = "13.0",
)

qemu_repository(
    name = "qemu_v5.2.0",
    version = "5.2.0",
)

http_archive(
    name = "rust_base64",
    build_file_content = """
load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")
rust_library(
    name = "base64",
    srcs = glob(["**/*.rs"]),
    visibility = ["//visibility:public"],
)
""",
    sha256 = "904dfeac50f3cdaba28fc6f57fdcddb75f49ed61346676a78c4ffe55877802fd",
    strip_prefix = "base64-0.13.0",
    type = "tar.gz",
    url = "https://crates.io/api/v1/crates/base64/0.13.0/download",
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
    name = "rust_json",
    build_file_content = """
load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")
rust_library(
    name = "json",
    srcs = glob(["**/*.rs"]),
    visibility = ["//visibility:public"],
)
""",
    sha256 = "078e285eafdfb6c4b434e0d31e8cfcb5115b651496faca5749b88fafd4f23bfd",
    strip_prefix = "json-0.12.4",
    type = "tar.gz",
    url = "https://crates.io/api/v1/crates/json/0.12.4/download",
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
)
""",
    sha256 = "4d58d1b70b004888f764dfbf6a26a3b0342a1632d33968e4a179d8011c760614",
    strip_prefix = "libc-0.2.80",
    type = "tar.gz",
    url = "https://crates.io/api/v1/crates/libc/0.2.80/download",
)
