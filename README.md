# Snapmount

Snapmount is a tool to create and mount backup snapshots.

As part of my backup strategy, I create short-lived LVM snapshots of my filesytems and mount them to a temporary location.
The backup program is run on these files, thus ensuring they are not being modified during the backup.
Once this is done, the LVM snapshots are unmounted and destroyed.

## Installation

You may install Snapmount locally by running

```console
$ cargo install --git https://github.com/vivienm/snapmount.git
```

## Configuration

Snapmount reads a configuration file (by default `/etc/snapmount/config.toml`) that describes the filesystems to be snapshotted and mounted.

```toml
# The list of LVM snapshots to create.
[[snapshots]]
  type = "lvm"
  source = "/dev/mapper/lvm-root"

  # Snaphot LV name (LV path will be `/dev/mapper/lvm-root`).
  name = "root-snap"

  # Size of the LVM snapshot. Defaults to 1G.
  size = "2G"


# The list of filesystems to be mounted, given in mounting order.
[[mounts]]
  source = "/dev/mapper/lvm-root-snap"

  # Mount directory, relative to the toplevel mount directory.
  target = "/"


[[mounts]]
  source = "/boot"

  # By default, `target` is the same as `source`,
  # so this one will be mounted to `$MOUNT_DIR/boot`.
  #target = "/boot"

  # Mount options.
  options = ["bind", "ro"]

[[mounts]]
  # Mount type.
  type = "efivarfs"
  source = "/sys/firmware/efi/efivars"
  options = ["nosuid", "noexec", "nodev"]

  # Mount this entry if and only if `source` exists.
  # Here, this allows using the same configuration file
  # with both BIOS and EFI boot loaders.
  if_exists = true
```

An advanced example of configuration file is given in [`examples/config.toml`](examples/config.toml).

## Usage

The command `snapmount mount <TARGET_DIR>` will create the snapshots and mount all entries defined in the configuration file.

```console
$ sudo snapmount mount /mnt
[INFO ] Creating snapshot lvm-root-snap of /dev/mapper/lvm-root
[INFO ] Mounting /dev/mapper/lvm-root-snap to /mnt
[INFO ] Mounting /boot to /mnt/boot
[INFO ] Mounting /sys/firmware/efi/efivars to /mnt/sys/firmware/efi/efivars
```

You can then run your backup program on the frozen rootfs `/mnt`.
Once this is done, run `snapmount unmount /mnt` to unmount everything and delete backup snapshots:

```console
$ sudo snapmount unmount /mnt
[INFO ] Unmounting /mnt/sys/firmware/efi/efivars
[INFO ] Unmounting /mnt/boot
[INFO ] Unmounting /mnt
[INFO ] Removing snapshot /dev/mapper/lvm-root-mnt
```

Run `snapmount --help` to list all options and commands.
