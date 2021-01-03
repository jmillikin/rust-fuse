load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

def _platform_transition_impl(settings, attr):
    return {
        "//command_line_option:platforms": [attr.platform],
    }

_platform_transition = transition(
    implementation = _platform_transition_impl,
    inputs = [],
    outputs = [
        "//command_line_option:platforms",
    ],
)

def _initrd(ctx):
    initrd = ctx.actions.declare_file(ctx.attr.name + ".cpio.gz")
    ctx.actions.run(
        outputs = [initrd],
        inputs = [
            ctx.file.busybox,
            ctx.file._test_runner,
        ],
        executable = ctx.executable._build_initrd,
        arguments = [
            ctx.file.busybox.path,
            ctx.file._test_runner.path,
            initrd.path,
        ],
    )
    return DefaultInfo(
        files = depset([initrd]),
    )

initrd = rule(
    implementation = _initrd,
    attrs = {
        "busybox": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "platform": attr.label(mandatory = True),
        "_test_runner": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = _platform_transition,
            default = "//fuse/tests:test_runner",
        ),
        "_build_initrd": attr.label(
            executable = True,
            cfg = "host",
            default = "//fuse/tests:build_initrd",
        ),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
)

def _transition_fileset(ctx):
    return DefaultInfo(
        files = depset(ctx.files.srcs),
    )

transition_fileset = rule(
    implementation = _transition_fileset,
    attrs = {
        "platform": attr.label(mandatory = True),
        "srcs": attr.label_list(
            cfg = _platform_transition,
            allow_files = True,
        ),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
)

def testcase_targets(cpus):
    testcase_names = [
        testcase[len("testcases/"):-len("/test_client.rs")]
        for testcase in native.glob(["testcases/*/test_client.rs"])
    ]

    for name in testcase_names:
        rust_binary(
            name = "testcases/{}/test_client".format(name),
            srcs = ["testcases/{}/test_client.rs".format(name)],
            deps = ["@rust_libc//:libc"],
        )
        rust_binary(
            name = "testcases/{}/test_server".format(name),
            srcs = ["testcases/{}/test_server.rs".format(name)],
            deps = ["//fuse"],
        )

    for cpu in cpus:
        srcs = []
        for name in testcase_names:
            srcs.extend([
                "testcases/{}/test_client".format(name),
                "testcases/{}/client_stdout.txt".format(name),
                "testcases/{}/test_server".format(name),
                "testcases/{}/server_stdout.txt".format(name),
            ])
        transition_fileset(
            name = "{}/testcases".format(cpu),
            platform = ":{}/platform".format(cpu),
            srcs = srcs,
        )

def _busybox_multiarch(ctx):
    ctx.file("WORKSPACE", "workspace(name = {name})\n".format(name = repr(ctx.name)))
    ctx.file("BUILD.bazel", "exports_files(glob(['**/*']))")

    ctx.download(
        url = ["https://busybox.net/downloads/binaries/1.31.0-defconfig-multiarch-musl/busybox-armv7l"],
        output = "busybox-armv7l",
        sha256 = "cd04052b8b6885f75f50b2a280bfcbf849d8710c8e61d369c533acf307eda064",
        executable = True,
    )
    ctx.download(
        url = ["https://busybox.net/downloads/binaries/1.31.0-defconfig-multiarch-musl/busybox-x86_64"],
        output = "busybox-x86_64",
        sha256 = "51fcb60efbdf3e579550e9ab893730df56b33d0cc928a2a6467bd846cdfef7d8",
        executable = True,
    )

busybox_multiarch = repository_rule(_busybox_multiarch)
