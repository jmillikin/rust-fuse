package(
    default_visibility = ["//build/testutil:__pkg__"],
)

_LINUX_CPUS = [
    "x86_64",
]

_FREEBSD_CPUS = [
    "x86_64",
]

platform(
    name = "macos",
    constraint_values = [
        "@platforms//os:macos",
        "@platforms//cpu:x86_64",
    ],
)

[platform(
    name = "linux_{}".format(cpu),
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:{}".format(cpu),
    ],
) for cpu in _LINUX_CPUS]

constraint_setting(name = "freebsd_version")

constraint_value(
    name = "freebsd_v12",
    constraint_setting = ":freebsd_version",
)

constraint_value(
    name = "freebsd_v13",
    constraint_setting = ":freebsd_version",
)

[platform(
    name = "freebsd_v12_{}".format(cpu),
    constraint_values = [
        ":freebsd_v12",
        "@platforms//os:freebsd",
        "@platforms//cpu:{}".format(cpu),
    ],
) for cpu in _FREEBSD_CPUS]

[platform(
    name = "freebsd_v13_{}".format(cpu),
    constraint_values = [
        ":freebsd_v13",
        "@platforms//os:freebsd",
        "@platforms//cpu:{}".format(cpu),
    ],
) for cpu in _FREEBSD_CPUS]
