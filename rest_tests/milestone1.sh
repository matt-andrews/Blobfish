#!/usr/bin/env bash
set -euo pipefail

bash run.sh

until [ "$(curl -s -o /dev/null -w "%{http_code}" localhost:31415/readyz)" == "200" ]; do
  echo "Waiting for 200 response..."
  sleep 5
done

#first put is create
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 201 ]; then
  echo "Failed 1: $status_code"
fi

#second put is ok
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 200 ]; then
  echo "Failed 2: $status_code"
fi

#get returns all
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" -ne "" ]; then
  echo "Failed 3: $body"
fi

#delete returns 204
status_code=$(curl -X DELETE -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 204 ]; then
  echo "Failed 4: $status_code"
fi

#get returns none
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" -ne "" ]; then
  echo "Failed 5: $body"
fi

#bash restart.sh

#get still returns none after restart
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" -ne "" ]; then
  echo "Failed 6: $body"
fi

#put now creates
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 201 ]; then
  echo "Failed 7: $status_code"
fi

#bash restart.sh

#bucket survives restart
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" -ne "" ]; then
  echo "Failed 8: $body"
fi

#bash down.sh