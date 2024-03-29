#! /bin/bash

export LANG=en_US.UTF8

exec &> >(tee /tmp/pika-installer-gtk4-log)

if [[ -f /tmp/pika-installer-gtk4-target-manual.txt ]]
then
    sudo /usr/lib/pika/pika-installer-gtk4/scripts/manual-partition-install.sh
else
    if [[ -f /tmp/pika-installer-gtk4-target-auto.txt ]]
    then
        sudo /usr/lib/pika/pika-installer-gtk4/scripts/automatic-partition-install.sh
    else
        echo "critical installer error" && exit 1 && touch /tmp/pika-installer-gtk4-fail.txt
    fi
fi
