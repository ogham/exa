#!/bin/bash

if [[ ! -d "/vagrant" ]]; then
    echo "This script should be run in the Vagrant environment"
    exit 1
fi

if [[ $EUID -ne 0 ]]; then
    echo "This script should be run as root"
    exit 1
fi

source "/vagrant/devtools/dev-fixtures.sh"


# create our test users

if id -u $FIXED_USER &>/dev/null; then
    echo "Normal user already exists"
else
    echo "Creating normal user"
    useradd $FIXED_USER
fi

if id -u $FIXED_LONG_USER &>/dev/null; then
    echo "Long user already exists"
else
    echo "Creating long user"
    useradd $FIXED_LONG_USER
fi


# locale generation

# remove most of this file, it slows down locale-gen
if grep -F -q "en_GB.UTF-8 UTF-8" /var/lib/locales/supported.d/en; then
    echo "Removing existing locales"
    echo "en_US.UTF-8 UTF-8" > /var/lib/locales/supported.d/en
fi

# uncomment these from the config file
if grep -F -q "# fr_FR.UTF-8" /etc/locale.gen; then
    sed -i '/fr_FR.UTF-8/s/^# //g' /etc/locale.gen
fi
if grep -F -q "# ja_JP.UTF-8" /etc/locale.gen; then
    sed -i '/ja_JP.UTF-8/s/^# //g' /etc/locale.gen
fi

# only regenerate locales if the config files are newer than the locale archive
if [[ ( /var/lib/locales/supported.d/en -nt /usr/lib/locale/locale-archive ) || \
      ( /etc/locale_gen                 -nt /usr/lib/locale/locale-archive ) ]]; then
    locale-gen
else
    echo "Locales already generated"
fi
