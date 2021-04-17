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

_CHECKSUMS = {
  "amd64/amd64/12.2-RELEASE/base.txz": "8bd49ce35c340a04029266fbbe82b1fdfeb914263e39579eecafb2e67d00693a",
  "amd64/amd64/12.2-RELEASE/kernel.txz": "729584a21f564cf9c1fa7d4a85ab6fa00a8c5370207396fa95d242b0bef750cb",
}

_BUILD = """
filegroup(
  name = "freebsd",
  srcs = glob(
    ["**/*"],
    exclude = [
      "BUILD.bazel",
      "WORKSPACE",
      "base.tar.xz",
      "kernel.tar.xz",
    ],
  ),
  visibility = ["//visibility:public"],
)
"""

def _freebsd_repository(ctx):
  ctx.file("WORKSPACE", "workspace(name = {})\n".format(repr(ctx.name)))
  ctx.file("BUILD.bazel", _BUILD)

  base_filename = "{}/{}-RELEASE/base.txz".format(ctx.attr.platform, ctx.attr.version)
  kernel_filename = "{}/{}-RELEASE/kernel.txz".format(ctx.attr.platform, ctx.attr.version)

  ctx.download(
    url = ["https://download.freebsd.org/ftp/releases/" + base_filename],
    output = "base.tar.xz",
    sha256 = _CHECKSUMS[base_filename],
  )

  ctx.download(
    url = ["https://download.freebsd.org/ftp/releases/" + kernel_filename],
    output = "kernel.tar.xz",
    sha256 = _CHECKSUMS[kernel_filename],
  )

  rc = ctx.execute(
    [
      "tar",
      "-xf",
      "kernel.tar.xz",
      "boot/kernel/kernel",
      "boot/kernel/cuse.ko",
      "boot/kernel/fusefs.ko",
      "boot/kernel/virtio_console.ko",
    ],
    quiet = False,
  )
  if rc.return_code != 0:
    fail("tar -xf kernel.tar.xz")

  rc = ctx.execute(
    [
      "tar",
      "-xf",
      "base.tar.xz",
      "boot/loader_simp.efi",
      "rescue/mt",
      "rescue/init",
      "rescue/rescue",
      "rescue/sh",
      "lib/libc.so.7",
      "lib/libgcc_s.so.1",
      "libexec/ld-elf.so.1",
    ],
    quiet = False,
  )
  if rc.return_code != 0:
    fail("tar -xf base.tar.xz")

freebsd_repository = repository_rule(
	implementation = _freebsd_repository,
	attrs = {
		"platform": attr.string(
			mandatory = True,
			values = [
        "amd64/amd64",
      ],
		),
		"version": attr.string(
			mandatory = True,
			values = [
        "12.2",
      ],
		),
	}
)
