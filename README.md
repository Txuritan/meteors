# Meteors

A (somewhat) simple [Archive of Our Own](https://archiveofourown.org/) HTML download viewer.

## Usage

Meteors is quite simple, with only two major command flags: `compress` and `trackers`.

When `compress` is active, data files will be compressed using [gzip](https://en.wikipedia.org/wiki/Gzip), which allows for up to a 3 times file size reduction.
<br>
Note, as of right now tracker scripts cannot be removed from a compressed file, be sure to run with `trackers` enabled before running this.

The `trackers` flag may need to be activated just to read a download, this is due to XHTML not being valid XML when scripts are used, enabling this will attempt to remove these scripts.
<br>
**WARNING** This could destroy the downloads, be sure to do backups. [For this reason](https://stackoverflow.com/a/1732454/4833195).


## Releases

Meteors is developed using the latest Rust stable, but is built using nightly.
This is so it can use `-Z build-std` to try and trim down the resulting binary.
