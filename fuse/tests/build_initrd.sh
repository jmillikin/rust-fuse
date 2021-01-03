#!/bin/sh
set -eu

busybox_path="$1"
test_runner_path="$2"
initrd_path="$3"

mkdir initrd-dir
cd initrd-dir

mkdir bin dev etc etc/init.d proc rust-fuse rust-fuse/testfs sys
cp -L "../${busybox_path}" bin/busybox
ln -s busybox bin/init
ln -s busybox bin/sh

cp -L "../${test_runner_path}" rust-fuse/test_runner

cat >etc/inittab <<EOF
::sysinit:/etc/init.d/rcS
ttyS0::respawn:/rust-fuse/test_runner
EOF
cat >etc/init.d/rcS <<EOF
#!/bin/sh
/bin/busybox mount proc -t proc /proc
/bin/busybox mount sysfs -t sysfs /sys
/bin/busybox mdev -s
EOF

chmod +x bin/* etc/init.d/rcS

find . -print0 | cpio --null -ov --format=newc | gzip -9 > "../${initrd_path}"
