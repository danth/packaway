# Packaway

Packaway is a substitution server for the local Nix store.

It's essentially a reimplementation of
[nix-serve](https://github.com/edolstra/nix-serve#readme) in Rust.

Packaway directly accesses the Nix database, and includes a Rust implementation
of the NAR file format to maximize performance.

## Setup

Add the flake `github:danth/packaway` to your server's configuration, and
import `nixosModules.packaway` from it.

Set `services.packaway.enable` to `true` to enable the service.

Generate a signing key by running
`nix-store --generate-binary-cache-key your.host.name /path/to/secret/key /path/to/public/key`,
then set `services.packaway.secretKey` to the path to the secret key.

Packaway runs on port 19082. You should either expose this port by adding it to
`networking.firewall.allowedTCPPorts`, or preferably use a reverse proxy to
serve it over HTTPS.

Once you've exposed the port, that's it! Rebuild the server and Packaway should
be available.

On the client, add the public key you generated to
`nix.settings.trusted-public-keys`, and finally add `http(s)://your.host.name/`
to `nix.settings.substituters`.
