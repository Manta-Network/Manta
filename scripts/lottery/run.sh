#!/bin/bash

while true; do
  # Get current UTC datetime
  datetime=$(date -u +"%Y%m%d%H%M%S")

  # Run the node command and redirect output to a new file
  node get_testnet_state.js >"logs/${datetime}.txt"

  # Wait for 12 seconds
  sleep 12
done
