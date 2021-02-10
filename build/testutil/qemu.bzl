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
  "v5.2.0/pc-bios/edk2-x86_64-code.fd.bz2": "8d9af6d88f51cfb6732a2542fefa50e7d5adb81aa12d0b79342e1bc905a368f1",
}

_BUILD = """
filegroup(
  name = "qemu",
  srcs = glob(
    ["**/*"],
    exclude = ["BUILD.bazel", "WORKSPACE"],
  ),
  visibility = ["//visibility:public"],
)
"""

def _qemu_repository(ctx):
  ctx.file("WORKSPACE", "workspace(name = {})\n".format(repr(ctx.name)))
  ctx.file("BUILD.bazel", _BUILD)

  edk2_filename = "v{}/pc-bios/edk2-x86_64-code.fd.bz2".format(ctx.attr.version)

  ctx.download(
    url = ["https://github.com/qemu/qemu/raw/" + edk2_filename],
    output = "pc-bios/edk2-x86_64-code.fd.bz2",
    sha256 = _CHECKSUMS[edk2_filename],
  )

  ctx.execute(["bzip2", "-d", "pc-bios/edk2-x86_64-code.fd.bz2"])

qemu_repository = repository_rule(
  implementation = _qemu_repository,
  attrs = {
    "version": attr.string(
      mandatory = True,
      values = [
        "5.2.0",
      ],
    ),
  }
)
