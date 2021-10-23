# rustypw
rpw - rusty password manager is basically a small cli password manager.

The passwords are stored encrypted with AES256 in CBC mode with a nonce iv. The
key is generated with argon2 using a randomly generated salt and password.

## Build
There are some simple wrappers around cargo commands. `make`
to build and `make install` to build a release version and make
a link to `realpath ~/bin` which obviously requires realpath :)

If you want to do something else than that `make release` builds
a release build for you to do with as you please.
