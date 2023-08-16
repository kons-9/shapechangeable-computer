#!/bin/bash
status=1
while [ $status -ne 0 ];
do
  cargo espflash flash --monitor --release --example estimate_coordinate
  status=$?
done

