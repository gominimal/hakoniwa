# Permission denied

## Problem

Hakoniwa runs as an unprivileged user and requires the **Linux namespaces** feature.
But this feature is restricted on some distros.

```console
$ hakoniwa run
hakoniwa: ... => Operation not permitted (os error 1)
```

## Solution

### Linux Hardened Kernel

Enable the sysctl setting `unprivileged_userns_clone` to allow normal users to run
unprivileged containers:

```console
$ sudo sysctl -w kernel.unprivileged_userns_clone=1
kernel.unprivileged_userns_clone = 1
```

### AppArmor

Turn off AppArmor user namespace creation restrictions:

```console
$ sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
kernel.apparmor_restrict_unprivileged_userns=0
```

## Links

- [Restricted unprivileged user namespaces are coming to Ubuntu 23.10](https://ubuntu.com/blog/ubuntu-23-10-restricted-unprivileged-user-namespaces)
- [AppArmor - Ubuntu Server documentation](https://documentation.ubuntu.com/server/how-to/security/apparmor/index.html)
- [unprivileged_userns_restriction](https://gitlab.com/apparmor/apparmor/-/wikis/unprivileged_userns_restriction)
