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
[mountpoint]
  # Toplevel mount directory.
  # All mountpoints will be mounted under this location.
  path = "/var/run/snapmount"

  # If true, the toplevel root directory is created/deleted by Snapmount.
  # Defaults to false.
  create = true

# The list of filesystems to be snapshotted and mounted,
# given in the right order for mounting them.
[[mounts]]
  type = "lvm"
  source = "/dev/mapper/lvm-root"

  # Mount directory, relative to `mountpoint.path`.
  # In this example, the snapshout will be mounted at `/var/run/snapmount`.
  target = "/"

  # LVM snapshotting settings.
  [mounts.snapshot]
    # Name of the LVM snapshot to be created.
    lv_name = "lvm-root-snapmount"

    # Size of the LVM snapshot. Defaults to 1G.
    #size = "1G"

[[mounts]]
  type = "lvm"
  source = "/dev/mapper/lvm-home"

  # Will be mounted at `/var/run/snapmount/home`.
  target = "/home"
  [mounts.snapshot]
    lv_name = "lvm-home-snapmount"
    size = "2G"

[[mounts]]
  # Arbitrary directories may also be mounted with a simple bind mount.
  # This may come handy to backup data residing on non-LVM volumes.
  type = "bind"
  source = "/boot"

  # `target` defaults to `source` for bind mounts.
  # This directory will be mounted at `/var/run/snapmount/boot`.
  #target = "/boot"
```

It is possible to use environment variables to populate values inside the configuration file:

```toml
[mountpoint]
  path = "${MOUNT_DIR:-/var/run/snapmount}"

[[mounts]]
  type = "lvm"
  source = "/dev/mapper/lvm-root"
  target = "/"
  [mounts.snapshot]
    lv_name = "lvm-root-snapmount-${BACKUP_PROFILE}"
```

An example configuration file is given in [`examples/config.toml`](examples/config.toml).

## Usage

The command `snapmount mount` will create the snapshots and mount them in the location defined in the configuration file (here, `/var/run/snapmount`).

```console
$ sudo snapmount mount
[INFO ] Creating toplevel mount directory /var/run/snapmount
[INFO ] Creating snapshot lvm-root-snapmount of /dev/mapper/lvm-root
[INFO ] Mounting /dev/mapper/lvm-root-snapmount to /var/run/snapmount/
[INFO ] Creating snapshot lvm-home-snapmount of /dev/mapper/lvm-home
[INFO ] Mounting /dev/mapper/lvm-home-snapmount to /var/run/snapmount/home
[INFO ] Bind mounting /boot to /var/run/snapmount/boot
```

You can then run your backup program on the frozen rootfs `/var/run/snapmount`.
Once this is done, run `snapmount unmount` to unmount and delete all backup snapshots:

```console
$ sudo snapmount unmount
[INFO ] Unmounting /var/run/snapmount/boot
[INFO ] Unmounting /var/run/snapmount/home
[INFO ] Removing snapshot /dev/mapper/lvm-home-snapmount
[INFO ] Unmounting /var/run/snapmount
[INFO ] Removing snapshot /dev/mapper/lvm-root-snapmount
[INFO ] Removing toplevel mount directory /var/run/snapmount
```
