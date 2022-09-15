load("@rules_rust//rust:defs.bzl", "rust_test")

filegroup(
    name = "srcs",
    srcs = glob(
        ["**/*.rs"],
        exclude = ["*_test.rs"],
    ),
    visibility = ["//:__subpackages__"],
)

rust_test(
    name = "linux_test",
    srcs = [
        "linux.rs",
        "linux_test.rs",
    ],
    crate_features = ["std"],
    edition = "2018",
    target_compatible_with = [
        "@platforms//os:linux",
    ],
)

rust_test(
    name = "linux_test_nostd",
    srcs = [
        "linux.rs",
        "linux_test.rs",
    ],
    crate_root = "linux_test.rs",
    edition = "2018",
    target_compatible_with = [
        "@platforms//os:linux",
    ],
)
