load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repository_set")

_NIGHTLY_DATE = "2020-12-30"

_SHA256S = {
    "2020-12-30/rust-nightly-x86_64-apple-darwin": "2b5b885694d0d1a9bdd0473d9e2df1f2c6eac88986e3135e6573e1d71e7824dc",
    "2020-12-30/llvm-tools-nightly-x86_64-apple-darwin": "8aca7ddf73983bf2db4846721787547fed16c2ad4dc5c260f7f05f6b93cea8e7",
    "2020-12-30/rust-std-nightly-x86_64-apple-darwin": "17912a6a5aa56daeb0aed5fca8698bacc54950351d9f91989a524588e37e41ca",
    "2020-12-30/rust-std-nightly-armv7-unknown-linux-musleabihf": "c7176fe7fccd6ab71535ce1abf81ab71c8cfdffbaa0f51f71d1d13b7f4526f22",
    "2020-12-30/rust-std-nightly-x86_64-unknown-linux-musl": "3802d2c7271cdd3fc35921b0d9f999b9b34ac9d888b62085b976453a8b113700",
}

def _rust_repository_set(**kwargs):
  rust_repository_set(
    edition = "2018",
    iso_date = _NIGHTLY_DATE,
    rustfmt_version = "1.4.20",
    sha256s = _SHA256S,
    version = "nightly",
    **kwargs
  )

def rust_toolchains():
  _rust_repository_set(
      name = "rustc_armv7-unknown-linux-musleabihf_{}".format(_NIGHTLY_DATE),
      exec_triple = "armv7-unknown-linux-musleabihf",
      extra_target_triples = [
          "x86_64-unknown-linux-musl",
      ],
  )
  _rust_repository_set(
      name = "rustc_x86_64-unknown-linux-musl_{}".format(_NIGHTLY_DATE),
      exec_triple = "x86_64-unknown-linux-musl",
      extra_target_triples = [
          "armv7-unknown-linux-musleabihf",
      ],
  )
  _rust_repository_set(
      name = "rustc_x86_64-apple-darwin_{}".format(_NIGHTLY_DATE),
      exec_triple = "x86_64-apple-darwin",
      extra_target_triples = [
          "armv7-unknown-linux-musleabihf",
          "x86_64-unknown-linux-musl",
          "x86_64-unknown-freebsd",
      ],
  )
