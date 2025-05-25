# Tlenix

Custom x86_64 OS built upon the Linux kernel. Boots from a USB.

Written in pure Rust without the standard library or any dependencies on a C standard library.

# Programs

`initramfs_init`: Init program for `initramfs`. Sets up important things like devices and the filesystem. Starts `init` when finished.

`init`: Responsible for booting up the system. Starts up `mash`.

`mash`: **Ma**x's **Sh**ell. An extremely primitive command-line-interface shell.

# Setup Guide

Here's how to get Tlenix running on a USB of your own.

## Watch Out!

Stuff like drive partitioning can screw up your system if you don't know what you're doing! If you aren't confident, I recommend using a virtual machine as the host when setting up the USB.

Additionally, make sure nothing important is stored on your USB you're using to boot this, because it _will_ be irrevocably wiped.

This project is in its early stages and is _not thoroughly tested_... Follow these instructions at your own risk.

## Build a Fresh Kernel

### Get the Linux Kernel Source

```bash
git clone git://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git
cd linux
```

### Select Your Kernel Version

List the possible kernel versions:

```bash
git tag
```

If in doubt, go with the most recent longterm kernel release. You can view kernel releases [here](https://www.kernel.org/releases.html). In this example, I will use version `6.12`:

```bash
git checkout v6.12
```

### Clean Stale Artifacts

You technically don't have to do this if it's a fresh kernel download, but it's a good habit to get into:

```bash
make mrproper
```

### Copy Over the Kernel Configuration

```bash
cp <tlenix dir>/config/.config <kernel dir>/.config
```

### Build the Kernel

This can take a while.

```bash
make -j$(nproc) CC="gcc-13" KCFLAGS="-std=gnu11" 2>&1 | tee log
```

Your finished kernel is located at `arch/x86_64/boot/bzImage` within the Linux source code directory.

## Build the Tlenix Binaries

Switch to the Tlenix source code directory.

```bash
cargo build --all --release
```

Your completed binaries will be under the `target/x86_64-unknown-linux-none/release` directory.

## Create the `initramfs`

We need to create a root filesystem which is used during the boot process. It loads necessary drivers and modules, then passes on control, switching to the actual persistent root filesystem on the USB.

### Create the `initramfs` Directory

```bash
mkdir initramfs
```

### Create the Basic `initramfs` Structure

```bash
mkdir -v initramfs/{bin,dev,etc,lib,proc,sys,usr}
```

### Create Essential Device Nodes

```bash
sudo mknod initramfs/dev/console c 5 1
sudo mknod initramfs/dev/null c 1 3
```

### Copy Over the Tlenix `initramfs_init` Program

```bash
cp <tlenix directory>/target/x86_64-unknown-linux-none/release/initramfs_init initramfs/init
```

### Create the `initramfs` Archive

```bash
cd initramfs
find . | cpio -oH newc | gzip >../root.cpio.gz
```

## Partition USB

### Warning

_THIS WILL DELETE EVERYTHING ON YOUR USB! BE CAREFUL!_

Additionally, it is _CRUCIAL_ that you identify the correct device... if you accidentally pick the wrong device, e.g. your computer's _hard drive_, you're in big trouble!

### USB Prep

### Identify Your USB Name

Use `lsblk` to list all your block devices. If in doubt, run it once with your USB unplugged, then a second time with your USB plugged in, in order to see which device your USB is.

```shell
lsblk
```

In this guide, we'll just call this device `/dev/sdX`, but MAKE SURE you substitute `/dev/sdX` with your _actual USB device_!

Additionally, check to see if any USB partitions are already mounted under the `MOUNTPOINTS` column in `lsblk`. If they are, unmount them from the path listed under `MOUNTPOINTS`:

```shell
sudo umount <mount path>
```

### Create Partitions With `fdisk`

After this step, there's _no turning back_! This will wipe anything on your chosen device. Double-triple-quadruple check that you have the correct device name before continuing.

```bash
sudo fdisk /dev/sdX
```

Inside `fdisk`:

```
g (GPT partition table)
n (New partition)
    Size: +500M (500 MiB)
    t: (Choose partition type)
    Type: 1 (EFI System)
n (New partition)
    Size: <Enter> (Remaining space)
    Type: 23 (Linux root (x86-64))
w (Save changes)
```

### Format Partitions

```bash
sudo mkfs.vfat -F32 /dev/sdX1
sudo mkfs.ext4 /dev/sdX2
```

## Organize USB Structure

### Mount Partitions

```bash
sudo mkdir -pv /mnt/tlenix-usb/{boot,rootfs}
sudo mount /dev/sdX1 /mnt/tlenix-usb/boot
sudo mount /dev/sdX2 /mnt/tlenix-usb/rootfs
```

### Populate Boot Partition

```bash
sudo cp <path to bzImage> /mnt/tlenix-usb/boot/
sudo cp <path to root.cpio.gz> /mnt/tlenix-usb/boot/
```

### Install GRUB

```bash
sudo grub-install \
    --target=x86_64-efi \
    --boot-directory=/mnt/tlenix-usb/boot \
    --efi-directory=/mnt/tlenix-usb/boot \
    /dev/sdX
```

### Configure GRUB

Copy over the GRUB config:

```bash
sudo cp <tlenix dir>/config/grub.cfg /mnt/tlenix-usb/boot/grub/grub.cfg
```

Optionally, you can edit the configuration:

```bash
sudoedit /mnt/tlenix-usb/boot/grub/grub.cfg
```

You can choose your own font. In the place of `TER16x32` in the config above, you can choose from any of the following:

- `MINI_4x6`
- `6x8`
- `6x10`
- `6x11`
- `7x14`
- `ACORN_8x8`
- `PEARL_8x8`
- `SUN8x16`
- `10x18`
- `SUN12x22`
- `TER16x32`

## Set Up the Root Filesystem

This is the filesystem which you actually use when running Tlenix.

### Create Minimal Directory Structure

```bash
sudo mkdir -pv /mnt/tlenix-usb/rootfs/{bin,dev,etc,home,lib,proc,sbin,sys,usr,var}
```

### Create Essential Device Nodes

```bash
sudo mknod /mnt/tlenix-usb/rootfs/dev/console c 5 1
sudo mknod /mnt/tlenix-usb/rootfs/dev/null c 1 3
```

### Add `init` Program

```bash
sudo cp <tlenix dir>/target/x86_64-unknown-linux-none/release/init /mnt/tlenix-usb/rootfs/sbin/init
```

### Add `mash` Program

```bash
sudo cp <tlenix dir>/target/x86_64-unknown-linux-none/release/mash /mnt/tlenix-usb/rootfs/bin/mash
```

## QEMU Test

```bash
sudo qemu-system-x86_64 \
    -kernel /mnt/tlenix-usb/boot/bzImage \
    -initrd /mnt/tlenix-usb/boot/root.cpio.gz \
    -drive file=/dev/sdX,format=raw \
    -append "root=/dev/sdX2 console=ttyS0" \
    -nographic
```
