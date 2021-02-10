load(
    "@io_bazel_rules_rust//rust:rust.bzl",
    "rust_binary",
    "rust_library",
)
load(":testutil.bzl", "freebsd_rootfs", "linux_rootfs", "qemu_exec")

sh_binary(
    name = "build_linux_rootfs",
    srcs = ["build_linux_rootfs.sh"],
)

linux_rootfs(
    name = "linux_rootfs",
    busybox = select({
        "@platforms//cpu:armv7": "@busybox_multiarch//:busybox-armv7l",
        "@platforms//cpu:x86_64": "@busybox_multiarch//:busybox-x86_64",
    }),
    kernel = select({
        "@platforms//cpu:armv7": "@linux_kernel//:arch/arm/boot/bzImage",
        "@platforms//cpu:x86_64": "@linux_kernel//:arch/x86_64/boot/bzImage",
    }),
)

sh_binary(
    name = "build_freebsd_rootfs",
    srcs = ["build_freebsd_rootfs.sh"],
)

freebsd_rootfs(
    name = "freebsd_rootfs",
    freebsd = select({
        "@platforms//cpu:x86_64": "@freebsd_amd64_v12.2//:freebsd",
    }),
)

qemu_exec(
    name = "qemu_exec",
    cpu = select({
        "@platforms//cpu:armv7": "armv7",
        "@platforms//cpu:x86_64": "x86_64",
    }),
    os = select({
        "@platforms//os:linux": "linux",
        "@platforms//os:freebsd": "freebsd",
    }),
    rootfs = select({
        "@platforms//os:linux": ":linux_rootfs",
        "@platforms//os:freebsd": ":freebsd_rootfs",
    }),
)

rust_binary(
    name = "qemu_exec_helper",
    srcs = ["qemu_exec_helper.rs"],
    deps = [
        "@rust_base64//:base64",
        "@rust_json//:json",
    ],
)
