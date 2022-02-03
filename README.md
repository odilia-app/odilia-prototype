# Odilia

A proof of concept Linux screen reader, with minimal features.

## Status: prototype

We're breaking things daily.
This is not usable whatsoever, and has no real functionality.
Check back later for progress.

The following information is for developers and testers:

## Building

### AUR

AUR Git Rpository: `https://aur.archlinux.org/odilia`

### Manual

Run the followimg commands:

```
$ cargo build
$ sudo ./fix-permissions.sh
$ cargo run
```

You will need to restart to activate the `udev` and group permission changes the `fix-permissions.sh` file makes.

 
