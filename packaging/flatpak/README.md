# Flatpak packaging

`org.southstar.WebBrowser.yml` builds Southstar as a Flatpak on the GNOME 51
runtime (freedesktop 26.08) with the `rust-stable` SDK extension. The
runtime brings GTK 4, FFmpeg (VP8, VP9, Opus, Vorbis), OpenSSL, SDL2,
libseccomp, libavif and enchant; the manifest builds only uchardet and the
browser. One bundle runs on any distribution with Flatpak, Ubuntu 22.04
and newer included.

The build is offline. `cargo-sources.json` lists every crates.io package in
`Cargo.lock` with the lock's checksum, and flatpak-builder downloads and
verifies them before cargo runs. Regenerate it whenever `Cargo.lock`
changes:

    python3 packaging/flatpak/cargo-sources.py

## Build

    flatpak install --user flathub org.gnome.Sdk//51 \
        org.freedesktop.Sdk.Extension.rust-stable//26.08
    flatpak-builder --user --force-clean --repo=repo build-dir \
        packaging/flatpak/org.southstar.WebBrowser.yml
    flatpak build-bundle \
        --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
        repo southstar.flatpak org.southstar.WebBrowser

## Install a bundle

    flatpak install --user southstar.flatpak
    flatpak run org.southstar.WebBrowser

The runtime comes from Flathub, which the bundle names, so the first
install downloads it.

## Sandboxing

The renderer keeps its own seccomp filter and Landlock rules inside the
Flatpak sandbox, and neither needs user namespaces. The app may use the
network, the display, PulseAudio, the GPU and the Downloads folder. The
browser can open files elsewhere only after `flatpak override --user
--filesystem=PATH org.southstar.WebBrowser`.
