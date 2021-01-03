load(
    "@io_bazel_rules_rust//rust:rust.bzl",
    "rust_binary",
    "rust_library",
    "rust_test",
)
load(
    ":tests.bzl",
    "initrd",
    "testcase_targets",
)

sh_binary(
    name = "build_initrd",
    srcs = ["build_initrd.sh"],
)

initrd(
    name = "armv7/initrd",
    busybox = "@busybox_multiarch//:busybox-armv7l",
    platform = ":armv7/platform",
)

initrd(
    name = "x86_64/initrd",
    busybox = "@busybox_multiarch//:busybox-x86_64",
    platform = ":x86_64/platform",
)

rust_library(
    name = "frame_io",
    srcs = ["frame_io.rs"],
)

rust_binary(
    name = "test_runner",
    srcs = ["test_runner.rs"],
    deps = [":frame_io"],
)

_CPUS = [
    "armv7",
    "x86_64",
]

[platform(
    name = "{}/platform".format(cpu),
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:{}".format(cpu),
    ],
) for cpu in _CPUS]

[filegroup(
    name = "{}/fuse_test_data".format(cpu),
    srcs = [
        "@linux_kernel//:arch/{}/boot/bzImage".format(cpu),
        ":{}/initrd".format(cpu),
        ":{}/testcases".format(cpu),
    ],
) for cpu in _CPUS]

[rust_test(
    name = "{}/fuse_test".format(cpu),
    srcs = ["fuse_test.rs"],
    args = [
        "--test-threads=1",
        "--nocapture",
    ],
    data = ["{}/fuse_test_data".format(cpu)],
    deps = [
        ":frame_io",
        "@rust_diff//:diff",
    ],
) for cpu in _CPUS]

testcase_targets(_CPUS)
