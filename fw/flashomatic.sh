#!/bin/bash

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

theusb=
otherusb=

if [ -f .gdbinit ]; then
    echo "Moving .gdbinit"
    mv .gdbinit .gdbinit-bak
fi

while true; do
    echo "Please plug NEXT board in now..."
    until [[ ! -z $theusb && ! -z $otherusb ]]; do
        usblist=($(ls /dev/cu.usbmodem* 2>/dev/null))
        if [ "$1" != "noerase" ]; then
            theusb=${usblist[1]}
            otherusb=${usblist[0]}
        else
            theusb=${usblist[0]}
            otherusb=${usblist[1]}
        fi
    done

    echo $theusb

    if [ "$1" != "noerase" ]; then
        echo "Erasing..."
        python erase.py $theusb

        echo "Wait for LED blink and then unplug..."
        until [[ -z $theusb && -z $otherusb ]]; do
            usblist=($(ls /dev/cu.usbmodem* 2>/dev/null))
            theusb=${usblist[1]}
            otherusb=${usblist[0]}
        done

        echo "Please plug in board again..."
        until [[ ! -z $theusb && ! -z $otherusb ]]; do
            usblist=($(ls /dev/cu.usbmodem* 2>/dev/null))
            theusb=${usblist[0]}
            otherusb=${usblist[1]}
        done

        echo $theusb
    fi

    echo "Flashing..."
    arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/paranoids-hackme-1-fw <<EOF
tar ext $theusb
mon sw
at 1
load
EOF

    echo "Please unplug the board..."
    until [[ -z $theusb && -z $otherusb ]]; do
        usblist=($(ls /dev/cu.usbmodem* 2>/dev/null))
        theusb=${usblist[1]}
        otherusb=${usblist[0]}
    done

    echo "Please plug in board again..."
    until [[ ! -z $theusb && ! -z $otherusb ]]; do
        usblist=($(ls /dev/cu.usbmodem* 2>/dev/null))
        theusb=${usblist[1]}
        otherusb=${usblist[0]}
    done

    echo $theusb

    echo "Provisioning..."
    python provisioner.py $theusb

    echo "Please unplug the board, it is done..."
    until [[ -z $theusb && -z $otherusb ]]; do
        usblist=($(ls /dev/cu.usbmodem* 2>/dev/null))
        theusb=${usblist[1]}
        otherusb=${usblist[0]}
    done
done
