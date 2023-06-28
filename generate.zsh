#!/bin/env zsh

local locations=(LOCALNET_DOWNLEFT LOCALNET_DOWNRIGHT LOCALNET_UPLEFT LOCALNET_UPRIGHT)

for i in $locations; do
  sed -e "s/TEMPLATE/$i/g" ./src/bin/write_efuse_template.rs > ./src/bin/write_efuse_${(L)i}.rs
done
