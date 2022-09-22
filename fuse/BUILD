load("@rules_rust//rust:defs.bzl", "rust_doc", "rust_doc_test", "rust_library")

ALL_SRCS = glob(["**/*.rs"])

filegroup(
    name = "test_srcs",
    srcs = ["//fuse/internal:testutil.rs"],
    visibility = ["//fuse:__subpackages__"],
)

rust_library(
    name = "fuse",
    srcs = [src for src in ALL_SRCS if not src.endswith("_test.rs")] + [
        "//fuse/internal:fuse_kernel_util.rs",
        "//fuse/internal:fuse_kernel.rs",
        "//fuse/internal:mod.rs",
        "//fuse/os:srcs",
        "//fuse/protocol/access:access.rs",
        "//fuse/protocol/bmap:bmap.rs",
        "//fuse/protocol/create:create.rs",
        "//fuse/protocol/copy_file_range:copy_file_range.rs",
        "//fuse/protocol/cuse_init:cuse_init.rs",
        "//fuse/protocol/fallocate:fallocate.rs",
        "//fuse/protocol/flush:flush.rs",
        "//fuse/protocol/forget:forget.rs",
        "//fuse/protocol/fsync:fsync.rs",
        "//fuse/protocol/fsyncdir:fsyncdir.rs",
        "//fuse/protocol/fuse_init:fuse_init.rs",
        "//fuse/protocol/getattr:getattr.rs",
        "//fuse/protocol/getlk:getlk.rs",
        "//fuse/protocol/getxattr:getxattr.rs",
        "//fuse/protocol/ioctl:ioctl.rs",
        "//fuse/protocol/link:link.rs",
        "//fuse/protocol/listxattr:listxattr.rs",
        "//fuse/protocol/lookup:lookup.rs",
        "//fuse/protocol/lseek:lseek.rs",
        "//fuse/protocol/mkdir:mkdir.rs",
        "//fuse/protocol/mknod:mknod.rs",
        "//fuse/protocol/open:open.rs",
        "//fuse/protocol/opendir:opendir.rs",
        "//fuse/protocol/read:read.rs",
        "//fuse/protocol/readdir:readdir.rs",
        "//fuse/protocol/readlink:readlink.rs",
        "//fuse/protocol/release:release.rs",
        "//fuse/protocol/releasedir:releasedir.rs",
        "//fuse/protocol/removexattr:removexattr.rs",
        "//fuse/protocol/rename:rename.rs",
        "//fuse/protocol/rmdir:rmdir.rs",
        "//fuse/protocol/setattr:setattr.rs",
        "//fuse/protocol/setlk:setlk.rs",
        "//fuse/protocol/setxattr:setxattr.rs",
        "//fuse/protocol/statfs:statfs.rs",
        "//fuse/protocol/symlink:symlink.rs",
        "//fuse/protocol/unlink:unlink.rs",
        "//fuse/protocol/write:write.rs",
        "//fuse/server/io:srcs",
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
