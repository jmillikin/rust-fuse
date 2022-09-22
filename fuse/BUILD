load("@rules_rust//rust:defs.bzl", "rust_doc", "rust_doc_test", "rust_library")

ALL_SRCS = glob(["src/**/*.rs"])

filegroup(
    name = "test_srcs",
    srcs = ["//fuse/src/internal:testutil.rs"],
    visibility = ["//fuse:__subpackages__"],
)

rust_library(
    name = "fuse",
    srcs = [src for src in ALL_SRCS if not src.endswith("_test.rs")] + [
        "//fuse/src/internal:fuse_kernel_util.rs",
        "//fuse/src/internal:fuse_kernel.rs",
        "//fuse/src/internal:mod.rs",
        "//fuse/src/os:srcs",
        "//fuse/src/protocol/access:access.rs",
        "//fuse/src/protocol/bmap:bmap.rs",
        "//fuse/src/protocol/create:create.rs",
        "//fuse/src/protocol/cuse_init:cuse_init.rs",
        "//fuse/src/protocol/fallocate:fallocate.rs",
        "//fuse/src/protocol/flush:flush.rs",
        "//fuse/src/protocol/forget:forget.rs",
        "//fuse/src/protocol/fsync:fsync.rs",
        "//fuse/src/protocol/fsyncdir:fsyncdir.rs",
        "//fuse/src/protocol/fuse_init:fuse_init.rs",
        "//fuse/src/protocol/getattr:getattr.rs",
        "//fuse/src/protocol/getlk:getlk.rs",
        "//fuse/src/protocol/getxattr:getxattr.rs",
        "//fuse/src/protocol/ioctl:ioctl.rs",
        "//fuse/src/protocol/link:link.rs",
        "//fuse/src/protocol/listxattr:listxattr.rs",
        "//fuse/src/protocol/lookup:lookup.rs",
        "//fuse/src/protocol/lseek:lseek.rs",
        "//fuse/src/protocol/mkdir:mkdir.rs",
        "//fuse/src/protocol/mknod:mknod.rs",
        "//fuse/src/protocol/open:open.rs",
        "//fuse/src/protocol/opendir:opendir.rs",
        "//fuse/src/protocol/read:read.rs",
        "//fuse/src/protocol/readdir:readdir.rs",
        "//fuse/src/protocol/readlink:readlink.rs",
        "//fuse/src/protocol/release:release.rs",
        "//fuse/src/protocol/releasedir:releasedir.rs",
        "//fuse/src/protocol/removexattr:removexattr.rs",
        "//fuse/src/protocol/rename:rename.rs",
        "//fuse/src/protocol/rmdir:rmdir.rs",
        "//fuse/src/protocol/setattr:setattr.rs",
        "//fuse/src/protocol/setlk:setlk.rs",
        "//fuse/src/protocol/setxattr:setxattr.rs",
        "//fuse/src/protocol/statfs:statfs.rs",
        "//fuse/src/protocol/symlink:symlink.rs",
        "//fuse/src/protocol/unlink:unlink.rs",
        "//fuse/src/protocol/write:write.rs",
        "//fuse/src/server/io:srcs",
    ],
    crate_features = ["std"],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = select({
        "@platforms//os:freebsd": [
            "@rust_freebsd_errno//freebsd-errno",
        ],
        "@platforms//os:linux": [
            "@rust_linux_errno//linux-errno",
        ],
        "//conditions:default": [],
    }),
)

rust_doc(
    name = "fuse_doc",
    crate = ":fuse",
)

rust_doc_test(
    name = "fuse_doc_test",
    crate = ":fuse",
)
