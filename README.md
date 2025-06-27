# Tlenix

Custom x86_64 OS built upon the Linux kernel. The OS consists of Tlenix user programs and a custom Linux kernel. Boots from a USB.

Written in pure Rust without the standard library or any dependencies on a C standard library.

# Programs

`initramfs_init`: Init program for the `initramfs`. Sets up important things like devices and the filesystem. Starts `init` when finished.

`init`: Responsible for booting up the system.

`mash`: **Ma**x's **Sh**ell. An extremely primitive command-line-interface shell.

`cat`: Concatenates files and prints them to the standard output.

`clear`: Clear the terminal screen.

`hello`: Example Tlenix program. Generates a greeting.

`ls`: Lists the entries within a directory.

`printenv`: Prints the current environment variables along with their values.

# Setup Guide - Virtual Machine

First, make sure you have an `x86_64` QEMU virtual machine on your system (e.g., `qemu-system-x86` for Ubuntu, `qemu-desktop` for Arch, etc.).

Next, if you wish to build your own kernel, consult the "Build a Fresh Kernel" step of the real hardware setup guide below. If you wish to use the prebuilt kernel, no further action is needed.

To run Tlenix in a QEMU VM, simply execute `./vm-run` from the root directory of this repository!

# Setup Guide - Real Hardware

Here's how to get Tlenix running on a USB of your own.

## 0. Watch Out!

Stuff like drive partitioning can screw up your system if you don't know what you're doing! If you aren't confident, I recommend using a virtual machine as the host when setting up the USB.

Additionally, make sure nothing important is stored on your USB you're using to boot this, because it _will_ be irrevocably wiped.

This project is in its early stages and is _not thoroughly tested_... Follow these instructions at your own risk.

## 1. Get the Tlenix Source

Go to a convenient place on your computer and clone the Tlenix source:

```bash
git clone https://github.com/maxgmr/tlenix.git
```

## 2. Build a Fresh Kernel

An already-built kernel is already available at `./bzKernel`. It uses the default Tlenix-tuned kernel configuration from `./config/.config`.

If you wish to use the prebuilt kernel, you may skip this step.

Alternatively, if you wish to build your own kernel, you may follow the instructions below:

### Get the Linux Kernel Source

Go to a different convenient place on your computer and clone the Linux kernel source:

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

### Copy Finished Kernel to Tlenix Source Directory

```bash
cp arch/x86_64/boot/bzImage <tlenix dir>/bzImage
```

## 3. Partition USB

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

## 4. Configure Tlenix

Inside the Tlenix source directory, the `config/` directory contains some options for customizing your Tlenix installation.

### Changing the Terminal Font

You can choose your own terminal font. To do so, open up `config/grub.cfg` in your preferred text editor. Instead of the default `TER16x32` font, you can set `fbcon=font:` to any of the following:

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

## 5. Install Tlenix

Make sure you're in the Tlenix source directory. Here's a pre-install checklist:

- Your partitioned and formatted USB is plugged in.
- `config/grub.cfg` exists.
- `bzImage` exists and is in the top level of the Tlenix source directory.

Make sure your partitioned and formatted USB is plugged in. Once you're ready, run the installation script:

```bash
./usb-install
```

## 6. Boot Into Tlenix

With your USB plugged in to your computer, restart your machine. If you boot _back_ into your computer's normal operating system, you'll have to go into your computer's BIOS menu and try the following things one at a time, restarting and retrying after each step:

1. Make sure your USB drive is at the top of the boot priority. This is the most common reason why you aren't automatically booting into Tlenix!
2. Make sure Secure Boot is disabled.
3. Disable CSM.
4. Set boot mode to UEFI only, not legacy/BIOS.
5. Disable Fast Boot.
