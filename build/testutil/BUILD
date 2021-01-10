load(
    "@io_bazel_rules_rust//rust:rust.bzl",
    "rust_binary",
    "rust_library",
)
load(":testutil.bzl", "initrd", "qemu_exec")

sh_binary(
    name = "build_initrd",
    srcs = ["build_initrd.sh"],
)

config_setting(
    name = "cfg_armv7-linux",
    constraint_values = [
        "@platforms//cpu:armv7",
        "@platforms//os:linux",
    ],
)

config_setting(
    name = "cfg_x86_64-linux",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
    ],
)

initrd(
    name = "initrd",
    busybox = select({
        "cfg_armv7-linux": "@busybox_multiarch//:busybox-armv7l",
        "cfg_x86_64-linux": "@busybox_multiarch//:busybox-x86_64",
    }),
)

qemu_exec(
    name = "qemu_exec",
    kernel = select({
        "cfg_armv7-linux": "@linux_kernel//:arch/arm/boot/bzImage",
        "cfg_x86_64-linux": "@linux_kernel//:arch/x86_64/boot/bzImage",
    }),
)

rust_binary(
    name = "qemu_exec_helper",
    srcs = ["qemu_exec_helper.rs"],
    deps = [
        "@rust_json//:json",
    ],
)
