load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

def _initrd(ctx):
    initrd = ctx.actions.declare_file(ctx.attr.name + ".cpio.gz")
    ctx.actions.run(
        outputs = [initrd],
        inputs = [
            ctx.file.busybox,
            ctx.file._qemu_exec_helper,
        ],
        executable = ctx.executable._build_initrd,
        arguments = [
            ctx.file.busybox.path,
            ctx.file._qemu_exec_helper.path,
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
        "_qemu_exec_helper": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = "target",
            default = "//build/testutil:qemu_exec_helper",
        ),
        "_build_initrd": attr.label(
            executable = True,
            cfg = "host",
            default = "//build/testutil:build_initrd",
        ),
    },
)

def _qemu_exec(ctx):
    out = ctx.actions.declare_file(ctx.attr.name + ".sh")

    # actions.expand_template(template, output, substitutions, is_executable=False)
    ctx.actions.expand_template(
        template = ctx.file._qemu_exec_tmpl,
        output = out,
        substitutions = {},
        is_executable = True,
    )

    return DefaultInfo(
        executable = out,
        files = depset([out]),
        runfiles = ctx.runfiles(
            files = [
                ctx.file.kernel,
                ctx.file._initrd,
                ctx.file._qemu_exec_helper,
            ],
        ),
    )

qemu_exec = rule(
    implementation = _qemu_exec,
    executable = True,
    attrs = {
        "kernel": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "_initrd": attr.label(
            allow_single_file = True,
            default = "//build/testutil:initrd",
        ),
        "_qemu_exec_helper": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = "host",
            default = "//build/testutil:qemu_exec_helper",
        ),
        "_qemu_exec_tmpl": attr.label(
            allow_single_file = True,
            default = "//build/testutil:qemu_exec_tmpl.sh",
        ),
    },
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
