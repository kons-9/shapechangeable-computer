#!/bin/env zsh

local locations=(LOCALNET_DOWNLEFT LOCALNET_DOWNRIGHT LOCALNET_UPLEFT LOCALNET_UPRIGHT)

for i in $locations; do
  local destination=./src/bin/write_efuse_${(L)i}.rs
  if [[ -f $destination ]]; then
    echo "File $destination already exists. remove it!"
    rm $destination
  fi
  sed -e "s/TEMPLATE/$i/g" ./template/write_efuse_template.rs > $destination
done
