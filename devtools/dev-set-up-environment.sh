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


# set up locales

# uncomment these from the config file
sudo sed -i '/fr_FR.UTF-8/s/^# //g' /etc/locale.gen
sudo sed -i '/ja_JP.UTF-8/s/^# //g' /etc/locale.gen

sudo locale-gen
