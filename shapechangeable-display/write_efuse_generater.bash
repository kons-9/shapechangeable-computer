#!/bin/bash

HISTORY_MAC_ADDRESS_FILE="history_mac_address.txt"
TEMPLATE_FILE="template/write_efuse_template.rs"
TARGET_DIRECTORY="src/bin/"

# make file which can write efuse register by using efuse_write_template.rs

# ask user to whether is_root or not.
echo "Are you root? (y/n)"
read is_root
# check is valid input
if [ $is_root != "y" ] && [ $is_root != "n" ]; then
  echo "Invalid input: not consist of y or n"
  exit 1
fi
# convert is_root to binary
if [ $is_root == "y" ]; then
  is_root_bin="1"
else
  is_root_bin="0"
fi

# ask user to local location(select from [upleft, upright, downleft, downright])
echo "Where is your local location? (upleft, upright, downleft, downright)"
read local_location
# check is valid inputs
if [ $local_location != "upleft" ] && [ $local_location != "upright" ] && [ $local_location != "downleft" ] && [ $local_location != "downright" ]; then
  echo "Invalid input: not consist of upleft, upright, downleft, downright"
  exit 1
fi
# convert local location to binary
# upleft: 00, upright: 01, downleft: 10, downright: 11
if [ $local_location == "upleft" ]; then
  local_location_bin="00"
elif [ $local_location == "upright" ]; then
  local_location_bin="01"
elif [ $local_location == "downleft" ]; then
  local_location_bin="10"
elif [ $local_location == "downright" ]; then
  local_location_bin="11"
fi

# ask localnet ip address
echo "What is your localnet ip address? (write binary ip address)"
read localnet_ip
# check is valid ip address(upto 29bit)
# consist of 0 or 1?
if [[ $localnet_ip =~ ^[01]+$ ]]; then
  # check is upto 29bit?
  if [ ${#localnet_ip} -gt 29 ]; then
    echo "Invalid ip address: too long"
    exit 1
  fi
else
  echo "Invalid ip address: not consist of 0 or 1"
  exit 1
fi
# root ip consists of only 0?
if [ $is_root == "y" ]; then
  if [[ $localnet_ip =~ ^[1]+$ ]]; then
    echo "Invalid ip address: root ip consists of only 0"
    exit 1
  fi
else
  if [[ $localnet_ip =~ ^[0]+$ ]]; then
    echo "Invalid ip address: not root ip consists of only 0"
    exit 1
  fi
fi

# convert localnet ip address to 29bit
# if ip address is less than 29bit, add 0 to ip address
if [ ${#localnet_ip} -lt 29 ]; then
  localnet_ip_length=$((29 - ${#localnet_ip}))
  for i in $(seq 1 ${localnet_ip_length}) ; do
    export localnet_ip="0"$localnet_ip
  done
fi

# check 29bit ip address is valid
if [ ${#localnet_ip} -ne 29 ]; then
  echo "Invalid ip address: not consist of 29bit"
  exit 1
fi

# make mac address
# mac address is 32bit: `localnet ip address(29) | local location(2) | is_root(1)`
mac_address=$localnet_ip$local_location_bin$is_root_bin
if [ ${#mac_address} -ne 32 ]; then
  echo "Invalid mac address: not consist of 32bit"
  exit 1
fi
echo "mac address is $mac_address"

# check is valid mac address(32bit) compared to history_mac_address.txt
# if valid, write mac address to history_mac_address.txt
#
# if this file does not exist, make this file and write mac address to this file
if [ ! -e $HISTORY_MAC_ADDRESS_FILE ]; then
  mkdir -p .cache
  touch $HISTORY_MAC_ADDRESS_FILE
else
  # else, search this file
  # if mac address is already written in this file, exit
  if grep -q $mac_address $HISTORY_MAC_ADDRESS_FILE; then
    echo "mac address is already written in history_mac_address.txt"
    exit 1
  # else, write mac address to this file
  fi
fi

echo "mac address is valid!"
echo "generating efuse write file..."

# make efuse write file from template
# replace TEMPLATE in template file to mac address
sed -e "s/TEMPLATE/$mac_address/g" $TEMPLATE_FILE > $TARGET_DIRECTORY"efuse_write_"$mac_address".rs"
status=$?
if [ $status -ne 0 ]; then
  echo "generating efuse write file failed"
  exit 1
fi

echo "generated efuse write file!"

echo "Do you want to write efuse register? (y/n)"
read is_write
if [ $is_write == "y" ]; then
  echo "writing efuse registers..."
  # write efuse registers
  echo "cargo espflash flash --monitor --bin efuse_write_$mac_address --release"
  cargo espflash flash --monitor --bin efuse_write_$mac_address --release
  status=$?
  if [ $status -ne 0 ]; then
    while [ $status -ne 0 ]; do
      echo "writing efuse registers failed"
      echo "retry it? (y/n)"
      read is_retry
      if [ $is_retry == "y" ]; then
        echo "cargo espflash flash --monitor --bin efuse_write_$mac_address --release"
        cargo espflash flash --monitor --bin efuse_write_$mac_address --release
        status=$?
      else
        echo "writing efuse registers failed"
        exit 1
      fi
    done
  fi
  echo "Do you want to delete efuse write file? (y/n)"
  read is_delete
  if [ $is_delete == "y" ]; then
    echo "deleting efuse write file..."
    rm $TARGET_DIRECTORY"efuse_write_"$mac_address".rs"
  fi
fi
  echo "adding mac address to history_mac_address.txt..."
  echo $mac_address >> $HISTORY_MAC_ADDRESS_FILE
