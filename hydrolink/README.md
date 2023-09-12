# Nashira Deer // Hydrolink

An implementation of a Lavalink client made with tokio independent of the Discord library, used in production by Hydrogen.

## Features

- `lavalink-trace`: It always makes REST calls using the `trace` parameter which makes the error responses have the stacktrace that originated it.
- `rustls-webpki-roots`: Enables the use of Rustls with Webpki roots.
- `rustls-native-roots`: Enables the use of Rustls with native roots.
- `native-tls`: Enables the use of native-tls.
- `native-tls-vendored`: Enables the use of native-tls with vendored feature.

## Credits

Hydrolink is a project by Nashira Deer for use within Hydrogen, licensed under the MIT License.
