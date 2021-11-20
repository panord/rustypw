[![Latest Version](https://img.shields.io/crates/v/rustypw.svg)](https://crates.io/crates/rustypw/)
[![docs](https://docs.rs/daemonize/badge.svg)](https://docs.rs/rustypw)
# rustypw

rpw - rusty password manager is basically a small cli password manager.

The passwords are stored encrypted with AES256 in CBC mode with a nonce iv. The
key is generated with argon2 using a randomly generated salt and password.

rpw stores all files under `$HOME/.rpw.d`. Including encrypted password storage
and its configuration file.

# External Dependencies
rpw depends upon `pbcopy` for MacOS and `xclip` on Linux to copy passwords to
the users clipboard. So these are required for rpw to function.

## Configuration
```
# $HOME/rpw.d/config.json
{
	clear_copy_timeout = UINT # Clipboard is cleared after timeout
}
```
# Usage

```
$ rpw new --vault demo
Please choose vault password (hidden):
Verify vault password (hidden):
New vault demo created
$ rpw new --vault demo
Please choose vault password (hidden):
Verify vault password (hidden):
$ rpw open demo
Please enter vault password (hidden):
demo$ add runescape
Please enter new password (hidden):
demo$ list
Stored passwords
        runescape
demo$ get runescape
Clearing clipboard in 5 seconds
```
