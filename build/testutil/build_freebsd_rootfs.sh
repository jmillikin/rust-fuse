#!/bin/sh
set -eu

rootfs_path="$1"
qemu_exec_helper_path="$2"
freebsd_path="${PWD}/$3"

mkdir "${rootfs_path}/rust-fuse"
cp "${qemu_exec_helper_path}" "${rootfs_path}/rust-fuse"

cd "${rootfs_path}"
mkdir {bin,boot,boot/kernel,dev,etc,lib,libexec,rescue,sbin,tmp}
mkdir rust-fuse/test_sandbox

touch dev/.keep
touch rust-fuse/test_sandbox/.keep
touch tmp/.keep

cp -L "${freebsd_path}/rescue/rescue" rescue/
ln -s ../rescue/rescue bin/sh
ln -s ../rescue/rescue sbin/init

cp -L "${freebsd_path}"/boot/loader_simp.efi boot/
cp -L "${freebsd_path}"/boot/kernel/* boot/kernel/
cp -L "${freebsd_path}"/lib/* lib/
cp -L "${freebsd_path}"/libexec/* libexec/

cat >etc/rc <<EOF
#!/bin/sh
set +eu
/rescue/rescue kldload cuse
/rescue/rescue kldload fusefs
/rescue/rescue kldload virtio_console

/rescue/rescue mdconfig -a -t swap -s 256m -u 0
/rescue/rescue newfs -U md0
/rescue/rescue mount /dev/md0 /rust-fuse/test_sandbox

/rescue/rescue mdconfig -a -t swap -s 256m -u 1
/rescue/rescue newfs -U md1
/rescue/rescue mount /dev/md1 /tmp

/rust-fuse/qemu_exec_helper
EOF

chmod +x etc/rc
