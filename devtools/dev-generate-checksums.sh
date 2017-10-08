# This script generates the MD5SUMS and SHA1SUMS files.
# You’ll need to have run ‘dev-download-and-check-release.sh’ and
# ‘local-package-for-macos.sh’ scripts to generate the binaries first.

set +x
trap 'exit' ERR

cd /vagrant
rm -f MD5SUMS SHA1SUMS

echo -e "\n\033[4mValidating MD5 checksums...\033[0m"
md5sum exa-linux-x86_64 exa-macos-x86_64 | tee MD5SUMS

echo -e "\n\033[4mValidating SHA1 checksums...\033[0m"
sha1sum exa-linux-x86_64 exa-macos-x86_64 | tee SHA1SUMS
