# This script downloads the published versions of exa from GitHub and my site,
# checks that the checksums match, and makes sure the files at least unzip and
# execute okay.
#
# The argument should be of the form “0.8.0”, no ‘v’. That version was the
# first one to offer checksums, so it’s the minimum version that can be tested.

set +x
trap 'exit' ERR

exa_version=$1
if [[ -z "$exa_version" ]]; then
    echo "Please specify a version, such as '$0 0.8.0'"
    exit 1
fi


# Delete anything that already exists
rm -rfv "/tmp/${exa_version}-downloads"


# Create a temporary directory and download exa into it
mkdir "/tmp/${exa_version}-downloads"
cd "/tmp/${exa_version}-downloads"

echo -e "\n\033[4mDownloading stuff...\033[0m"
wget --quiet --show-progress "https://github.com/ogham/exa/releases/download/v${exa_version}/exa-macos-x86_64-${exa_version}.zip"
wget --quiet --show-progress "https://github.com/ogham/exa/releases/download/v${exa_version}/exa-linux-x86_64-${exa_version}.zip"

wget --quiet --show-progress "https://github.com/ogham/exa/releases/download/v${exa_version}/MD5SUMS"
wget --quiet --show-progress "https://github.com/ogham/exa/releases/download/v${exa_version}/SHA1SUMS"


# Unzip the zips and check the sums
echo -e "\n\033[4mExtracting that stuff...\033[0m"
unzip "exa-macos-x86_64-${exa_version}.zip"
unzip "exa-linux-x86_64-${exa_version}.zip"

echo -e "\n\033[4mValidating MD5 checksums...\033[0m"
md5sum -c MD5SUMS

echo -e "\n\033[4mValidating SHA1 checksums...\033[0m"
sha1sum -c SHA1SUMS


# Finally, give the Linux version a go
echo -e "\n\033[4mChecking it actually runs...\033[0m"
./"exa-linux-x86_64" --version
./"exa-linux-x86_64" --long

echo -e "\n\033[1;32mAll's lookin' good!\033[0m"
