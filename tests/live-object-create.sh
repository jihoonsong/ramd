#!/bin/bash

if [ "$1" == "sum" ]; then
    wasm_bytes=$(cat tests/wasms/live_object_sum_base64)
elif [ "$1" == "gcounter" ]; then
    wasm_bytes=$(cat tests/wasms/live_object_gcounter_base64)
else
    echo "Invalid argument. Please use 'sum' or 'gcounter'."
    exit 1
fi

curl --location '0.0.0.0:1319' \
--header 'Content-Type: application/json' \
--data '{
  "jsonrpc": "2.0",
  "method": "live_object_create",
  "params": {
      "request": {
          "wasm_bytes": "'"$wasm_bytes"'"
      }
  },
  "id": 1
}'
