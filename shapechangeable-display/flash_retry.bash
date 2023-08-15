#!/bin/bash
status=1
while [ $status -ne 0 ];
do
  cargo espflash flash --monitor --release --example read_efuse
  status=$?
done

