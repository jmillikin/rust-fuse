#!/bin/sh
set -eu

busybox_path="$1"
qemu_exec_helper_path="$2"
initrd_path="$3"

mkdir initrd-dir
cd initrd-dir

mkdir bin dev etc etc/init.d proc rust-fuse rust-fuse/test_sandbox sys tmp
cp -L "../${busybox_path}" bin/busybox
ln -s busybox bin/init
ln -s busybox bin/sh

cp -L "../${qemu_exec_helper_path}" rust-fuse/qemu_exec_helper

cat >etc/inittab <<EOF
::sysinit:/etc/init.d/rcS
ttyS0::respawn:/rust-fuse/qemu_exec_helper
EOF
cat >etc/init.d/rcS <<EOF
#!/bin/sh
/bin/busybox mount proc -t proc /proc
/bin/busybox mount sysfs -t sysfs /sys
/bin/busybox mount tmpfs -t tmpfs /tmp
/bin/busybox mdev -s
EOF

chmod +x bin/* etc/init.d/rcS

find . -print0 | cpio --null -ov --format=newc | gzip -9 > "../${initrd_path}"
