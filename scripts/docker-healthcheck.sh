#!/bin/bash
set -e

# Send a request to the health endpoint
STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health)

# Consider both 200 and 503 as healthy (503 means degraded but still functional)
if [ "$STATUS_CODE" -eq "200" ] || [ "$STATUS_CODE" -eq "503" ]; then
  exit 0
else
  exit 1
fi 