load("@io_bazel_rules_rust//rust:rust.bzl", "rust_test")

def rust_fuse_protocol_module():
    files = native.glob(["*.rs"])
    name = native.package_name()[len("fuse/src/protocol/"):]

    if name + "_test.rs" in files:
        rust_test(
            name = name + "_test",
            srcs = [name + "_test.rs"] + [
                "//fuse:test_srcs",
            ],
            size = "small",
            timeout = "short",
            crate = "//fuse",
            crate_features = [
                "std",
                "unstable_" + name,
            ],
            rustc_flags = ['--cfg=rust_fuse_test="{}_test"'.format(name)],
        )

    if name + "_interop_test.rs" in files:
        test_name = name + "_interop_test"
        if test_name not in native.existing_rules():
            rust_test(
                name = test_name,
                srcs = [name + "_interop_test.rs"],
                size = "medium",
                timeout = "short",
                crate_features = ["std"],
                deps = [
                    "//fuse",
                    "//fuse/src/internal:interop_testutil",
                    "@rust_libc//:libc",
                ],
            )
