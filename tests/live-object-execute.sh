#!/bin/bash

if [ "$1" == "sum" ]; then
    if [ -z "$2" ] || [ -z "$3" ]; then
        echo "Missing arguments for sum. Usage: $0 sum x y"
        exit 1
    fi

    x=$2
    y=$3
    args="{\\\"x\\\": $x, \\\"y\\\": $y}"
    live_object_id="67700b725575434de878141282f35a6d154688b608491b2a2539783ceef20996"
    method="sum"
elif [ "$1" == "gcounter" ]; then
    if [ -z "$2" ]; then
        echo "Missing argument for gcounter. Usage: $0 gcounter delta"
        exit 1
    fi
    
    delta=$2
    args="{\\\"delta\\\": $delta}"
    live_object_id="eabe86b1378265b7dc3416274f565a98ecd88147a291c6d5fcaaf344e85d9bc5"
    method="increment"
else
    echo "Invalid operation. Use 'sum' or 'gcounter'."
    exit 1
fi


curl --location '0.0.0.0:1319' \
--header 'Content-Type: application/json' \
--data '{
  "jsonrpc": "2.0",
  "method": "live_object_execute",
  "params": {
      "request": {
          "live_object_id": "'"$live_object_id"'",
          "method": "'"$method"'",
          "args": "'"$args"'"
      }
  },
  "id": 1
}'
