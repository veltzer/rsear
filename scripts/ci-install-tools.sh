#!/bin/bash  -eu
# Install the system libraries this repo's build links against and the
# external tools its tests shell out to. The canonical ci.yml runs this in
# every repo before `cargo build`; a repo that needs neither keeps this
# script as an explicit no-op. Keep it strict: anything that fails to
# install must fail the build here, not surface later as a confusing
# build or test failure.
set -euo pipefail

# Acquire::Retries because apt's default is 0: when the first mirror in
# /etc/apt/apt-mirrors.txt is unreachable there is no second attempt;
# Retries=3 lets apt fall through to archive.ubuntu.com.
apt_get() {
	sudo apt-get -o Acquire::Retries=3 "$@"
}

# rsear plays audio through cpal, whose alsa-sys build script probes
# pkg-config for the ALSA headers — without libasound2-dev the build fails
# before compiling a single Rust file.
#
# ALSA has to be present for the architecture the binary links against,
# which on the release job's aarch64 cross build is not the runner's own.
# TARGET is the triple the calling workflow step compiles for (ci.yml's
# build job exports its matrix target); it is unset in the test job, where
# the build is native. Any other triple is a hard error rather than a
# guess: a new target needs its own recipe here.
case "${TARGET:-}" in
"" | x86_64-unknown-linux-gnu)
	foreign=""
	multiarch=""
	;;
aarch64-unknown-linux-gnu)
	foreign="arm64"
	multiarch="aarch64-linux-gnu"
	;;
*)
	echo "ci-install-tools.sh: no ALSA recipe for target ${TARGET}" >&2
	exit 1
	;;
esac

if [[ -n "${foreign}" ]]; then
	# Foreign-architecture packages live on ports.ubuntu.com, which the
	# runner's amd64 archive mirrors do not carry, so the foreign
	# architecture gets its own source. The existing sources are pinned to
	# amd64 first: once a second architecture is enabled, apt asks every
	# unpinned source for that architecture's indexes too, and the amd64
	# mirrors have none to give.
	codename="$(sed -n 's/^VERSION_CODENAME=//p' /etc/os-release)"
	for f in /etc/apt/sources.list.d/*.sources; do
		[[ -f "${f}" ]] || continue
		if ! grep -q '^Architectures:' "${f}"; then
			sudo sed -i '/^Types:/a Architectures: amd64' "${f}"
		fi
	done
	if [[ -f /etc/apt/sources.list ]]; then
		sudo sed -i -E '/^deb +\[/! s/^deb +/deb [arch=amd64] /' /etc/apt/sources.list
	fi
	sudo tee "/etc/apt/sources.list.d/ports-${foreign}.sources" >/dev/null <<PORTS
Types: deb
URIs: http://ports.ubuntu.com/ubuntu-ports
Suites: ${codename} ${codename}-updates ${codename}-security
Components: main universe
Architectures: ${foreign}
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
PORTS
	sudo dpkg --add-architecture "${foreign}"
fi

apt_get update

if [[ -z "${foreign}" ]]; then
	apt_get install -y libasound2-dev
else
	# The foreign ALSA packages are unpacked into a private sysroot rather
	# than installed. `apt-get install libasound2-dev:arm64` drags in
	# libc6:arm64 and libgcc-s1:arm64, and dpkg's Multi-Arch: same rule
	# requires those to be byte-for-byte the same version as the native
	# ones. The hosted runner image ships a gcc-16-base newer than anything
	# the archive offers for arm64, so apt could only satisfy the rule by
	# downgrading the runner's own toolchain, which -y refuses (run
	# 34364098969: "Packages were downgraded and -y was used without
	# --allow-downgrades"). The cross build needs none of that: it links
	# against the aarch64 libc that gcc-aarch64-linux-gnu already brings
	# in, and only needs ALSA's headers, .pc file, and shared object, which
	# dpkg -x provides without touching the native package database.
	# libasound2t64 carries the real libasound.so.2.0.0; the -dev package
	# holds the headers, the pkg-config file, and the libasound.so symlink
	# the linker resolves -lasound through.
	sysroot="${HOME}/sysroot-${multiarch}"
	mkdir -p "${sysroot}/debs"
	(cd "${sysroot}/debs" && apt-get download "libasound2-dev:${foreign}" "libasound2t64:${foreign}")
	for deb in "${sysroot}"/debs/*.deb; do
		dpkg -x "${deb}" "${sysroot}"
	done

	# PKG_CONFIG_SYSROOT_DIR makes pkg-config prefix every -I and -L in
	# alsa.pc with the sysroot, and its presence is what tells the
	# pkg-config crate that cross probing for this target is allowed;
	# PKG_CONFIG_LIBDIR points it at the sysroot's .pc files instead of the
	# host's. Both are per target, so the native x86_64 build in the same
	# job graph is unaffected. Exported through GITHUB_ENV so the Build
	# step that follows sees them; outside Actions they are printed for the
	# caller to export by hand.
	target_var="${TARGET//-/_}"
	{
		echo "PKG_CONFIG_SYSROOT_DIR_${target_var}=${sysroot}"
		echo "PKG_CONFIG_LIBDIR_${target_var}=${sysroot}/usr/lib/${multiarch}/pkgconfig:${sysroot}/usr/share/pkgconfig"
	} >>"${GITHUB_ENV:-/dev/stderr}"
fi

# The tests synthesize audio through fluidsynth against the FluidR3 General
# MIDI soundfont at its packaged path (/usr/share/sounds/sf2/FluidR3_GM.sf2,
# the SOUNDFONT_PATH constant in src/audio.rs and tests/basic.rs).
apt_get install -y fluid-soundfont-gm
