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
load("//build:rust_toolchains.bzl", "rust_toolchains")

cc_toolchains()

rust_toolchains()
