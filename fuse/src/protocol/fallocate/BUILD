load("@io_bazel_rules_rust//rust:rust.bzl", "rust_test")
load("//fuse:src/protocol/common.bzl", "rust_fuse_protocol_module")

exports_files(glob(["*.rs"]))

rust_test(
	name = "fallocate_interop_test",
	srcs = ["fallocate_interop_test.rs"],
	size = "medium",
	timeout = "short",
	crate_features = ["std"],
	deps = [
		"//fuse",
		"//fuse/src/internal:interop_testutil",
		"@rust_libc//:libc",
	],
	target_compatible_with = [
		"@platforms//os:linux",
	],
)

rust_fuse_protocol_module()
