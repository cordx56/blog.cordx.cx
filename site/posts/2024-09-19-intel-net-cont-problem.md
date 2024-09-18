---
title: Workaround for unusable Intel I226-V Ethernet controller (igc) without rebooting
date: 2024-09-19 00:32
tags: [linux, network]
---

Recently, our laboratory assembled a new computer to use as a server, but after some time elapsed following booting, the Ethernet suddenly became nonfunctional.
In this article, I will show an automated workaround for this problem.

Note: I usually write articles in Japanese, but since few people have reported this problem and there are no articles showing a workaround even in English, I decided to write this article in English.
Please forgive my bad English.

## Environment

The environment in which we observed the problem is as follows:

- OS: Ubuntu Desktop 24.04.1
- Motherboard: ASUS ROG STRIX X670E-A GAMING WIFI
- CPU: AMD Ryzen 9 7950X3D

The motherboard's Ethernet controller, an Intel I226-V, has become dysfunctional.

## Related articles

I found some related articles, but they do not provide an appropriate workaround that works in our environment.

- [Network card (Intel Ethernet Controller I225-V, igc) keeps dropping after 1 hour on linux - solved with kernel param](https://www.reddit.com/r/buildapc/comments/xypn1m/network_card_intel_ethernet_controller_i225v_igc/)

## Detailed procedure to solve the problem without rebooting

Here, I will show the procedure I tried that works in our environment.
All commands shown below are executed in bash.

1. Obtain the Ethernet controller's device ID

```bash
$ lspci | grep I226-V | awk '{print $1}'
```

This command retrieves the I226-V's PCI device ID using the `lspci` command.

2. Identify the PCI device path

Second, identify the PCI device path of the Ethernet controller:

```bash
$ ls -d /sys/bus/pci/devices/* | grep ${intel_id}
```

This command lists all PCI device paths and filters them. Replace `${intel_id}` with your I226-V's PCI device ID.

3. Remove I226-V from system

This command requires root permission.

```bash
# echo 1 > ${device_path}/remove
```

`${device_path}` is obtained in step 2.

4. Rescan all PCI devices

This command also requires root permission.

```
# echo 1 > /sys/bus/pci/rescan
```

After a moment, the network will work again.

Note: I tried `modprobe -r igc` and `modprobe igc` and it not fixes the problem, but there is a possibility that these commands affect the result.

## Automate execution of these commands when the network is down

Here, I will show a script that automatically executes these procedures.

```bash
#!/bin/bash

function reset_device() {
    local intel_id=$(lspci | grep I226-V | awk '{print $1}')
    echo "Network device is ${intel_id}"
    local device_path=$(ls -d /sys/bus/pci/devices/* | grep "${intel_id}")
    echo "Resetting device..."
    echo 1 > "${device_path}/remove"
    echo 1 > /sys/bus/pci/rescan
    echo "Network device reset"
    sleep 300
}

while true; do
    ping -q -c 1 your_host_here || reset_device
    sleep 3
done
```

This script sends a ping to another machine, and if it returns an error, then it tries to reset the Ethernet controller.

## Conclusion

In this article, we confirmed that on Linux (Ubuntu 24.04.1), there is a problem with the Ethernet controller, Intel I226-V (igc), and we provided a workaround for this issue without rebooting.

**TL;DR**: ***BUY A NEW NETWORK CARD***
