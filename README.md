# tlenix

Custom x86_64 OS built upon the Linux kernel. Boots from a USB.

## Setup - USB UEFI Boot

### 0. Watch out!

Messing with stuff like the bootloader can screw up your system if you don't know what you're doing! If you aren't confident, I recommend using a virtual Linux machine as the host when setting up the USB.

This project is in its early stages and is _not thoroughly tested_... Follow these instructions at your own risk.

### 1. Partition the USB

Make 2 partitions:

1. EFI System partition (~512 MiB if you're not sure) formatted as FAT-32
2. Dedicate the rest to a Linux root (x86-64) partition formatted as anything that works with Linux (I use ext4).

### 2. Set up the USB for booting

In these examples, I'm using `/dev/sda1` as the name of the EFI System partition and `/dev/sda2` as the name of the Linux root partition- your device names will likely be different!

**Make sure** you have the right drive/partition names. I don't want you messing up your files!

Mount the root partition: `mount /dev/sda2 /mnt`

Make a place for the EFI partition to live: `mkdir -pv /mnt/boot/efi`

Mount the EFI partition: `mount /dev/sda1 /mnt/boot/efi`

### 3. Install GRUB on the drive

Make sure you get these arguments right- you don't want to overwrite your own GRUB config!

`grub-install --target=x86_64-efi --efi-directory=/mnt/boot/efi --boot-directory=/mnt/boot /dev/sda`

### 4. Copy the Kernel and Initramfs

The easiest thing to do is to simply use your current kernel and initramfs, but you can absolutely make your own if you prefer.

These examples assume your kernel image is named `vmlinuz-linux` and your initramfs is named `initramfs-linux.img`- adjust your file names accordingly.

`cp /boot/{vmlinuz-linux,initramfs-linux.img} /mnt/boot`

### 5. Configure GRUB

Create a new GRUB configuration at `/mnt/boot/grub/grub.cfg`. Here's an example:

```
set default=0
set timeout=30

menuentry "tlenix" {
    linux /boot/vmlinuz-linux root=/dev/sda2 rw
    initrd /boot/initramfs-linux.img
}
```

### 6. Create basic directory structures

The kernel needs these directories to do stuff:

`mkdir -pv /mnt/{bin,sbin,etc,lib,lib64,var,dev,proc,sys,run,tmp}`

Make some nodes that the kernel also needs:

/dev/console character device: `mknod -m 600 /mnt/dev/console c 5 1`

/dev/null character device: `mknod -m 666 /mnt/dev/null c 1 3`

### 7. Add tlenix init

Build the binary: `./build_init.sh`

Copy the binary to the USB: `cp init/init /mnt/sbin/`

### 8. Reboot and run!

If you don't boot into GRUB with `tlenek` as an option, make sure that your USB stick is at the top of boot priority in your computer's BIOS settings.
