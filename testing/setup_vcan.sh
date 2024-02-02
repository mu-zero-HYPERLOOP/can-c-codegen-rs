#!/usr/bin/bash

sudo ip link add dev can0 type vcan
sudo ip link set up can0

sudo ip link add dev can1 type vcan
sudo ip link set up can1
