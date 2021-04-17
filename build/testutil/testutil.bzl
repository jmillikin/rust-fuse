# Copyright 2021 John Millikin and the rust-fuse contributors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# SPDX-License-Identifier: Apache-2.0

load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")
load(":freebsd.bzl", _freebsd_repository = "freebsd_repository")
load(":qemu.bzl", _qemu_repository = "qemu_repository")

freebsd_repository = _freebsd_repository
qemu_repository = _qemu_repository

def _linux_rootfs(ctx):
    rootfs = ctx.actions.declare_directory(ctx.attr.name)
    ctx.actions.run(
        executable = ctx.executable._build_linux_rootfs,
        arguments = [
            rootfs.path,
            ctx.file.busybox.path,
            ctx.file.kernel.path,
            ctx.file._qemu_exec_helper.path,
        ],
        inputs = [
            ctx.file.busybox,
            ctx.file.kernel,
            ctx.file._qemu_exec_helper,
        ],
        outputs = [rootfs],
        mnemonic = "LinuxRootFilesystem",
    )
    return DefaultInfo(files = depset(direct = [rootfs]))

linux_rootfs = rule(
    implementation = _linux_rootfs,
    attrs = {
        "busybox": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "kernel": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "_qemu_exec_helper": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = "target",
            default = "//build/testutil:qemu_exec_helper",
        ),
        "_build_linux_rootfs": attr.label(
            executable = True,
            cfg = "host",
            default = "//build/testutil:build_linux_rootfs",
        ),
    },
)

def _freebsd_rootfs(ctx):
    rootfs = ctx.actions.declare_directory(ctx.attr.name)
    ctx.actions.run(
        executable = ctx.executable._build_freebsd_rootfs,
        arguments = [
            rootfs.path,
            ctx.file._qemu_exec_helper.path,
            ctx.files.freebsd[0].owner.workspace_root,
        ],
        inputs = ctx.files.freebsd + [
            ctx.file._qemu_exec_helper,
        ],
        outputs = [rootfs],
        mnemonic = "FreebsdRootFilesystem",
    )
    return DefaultInfo(files = depset(direct = [rootfs]))

freebsd_rootfs = rule(
    implementation = _freebsd_rootfs,
    attrs = {
        "freebsd": attr.label(
            allow_files = True,
            mandatory = True,
        ),
        "_qemu_exec_helper": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = "target",
            default = "//build/testutil:qemu_exec_helper",
        ),
        "_build_freebsd_rootfs": attr.label(
            executable = True,
            cfg = "host",
            default = "//build/testutil:build_freebsd_rootfs",
        ),
    },
)

def _qemu_exec(ctx):
    out = ctx.actions.declare_file(ctx.attr.name + ".sh")
    content = """#!/bin/sh
export RUST_FUSE_TEST_CPU={}
export RUST_FUSE_TEST_OS={}
export RUST_FUSE_TEST_ROOTFS={}
exec build/testutil/qemu_exec_helper "$@"
""".format(ctx.attr.cpu, ctx.attr.os, ctx.file.rootfs.short_path)
    ctx.actions.write(out, content, is_executable = True)

    return DefaultInfo(
        executable = out,
        files = depset([out]),
        runfiles = ctx.runfiles(
            files = [
                ctx.file.rootfs,
                ctx.file._qemu_exec_helper,
            ] + ctx.files._qemu,
        ),
    )

qemu_exec = rule(
    implementation = _qemu_exec,
    executable = True,
    attrs = {
        "rootfs": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "cpu": attr.string(
            mandatory = True,
        ),
        "os": attr.string(
            mandatory = True,
        ),
        "_qemu": attr.label(
            default = "@qemu_v5.2.0//:qemu",
        ),
        "_qemu_exec_helper": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = "host",
            default = "//build/testutil:qemu_exec_helper",
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
