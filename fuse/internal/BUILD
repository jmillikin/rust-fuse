exports_files(["testutil.rs"])

filegroup(
    name = "srcs",
    srcs = glob(
        ["*.rs"],
        exclude = ["testutil.rs"],
    ),
    visibility = ["//fuse:__subpackages__"],
)
