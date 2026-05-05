#!/usr/bin/env bash
set -euo pipefail

bash run.sh


 readiness_timeout=60
 readiness_interval=5
 readiness_start=$SECONDS

 until [ "$(curl -s -o /dev/null -w "%{http_code}" localhost:31415/readyz)" == "200" ]; do
   if (( SECONDS - readiness_start >= readiness_timeout )); then
     echo "Service did not become ready within ${readiness_timeout} seconds."
     exit 1
   fi
   echo "Waiting for 200 response..."
      sleep "$readiness_interval"
    done

#first put is create
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 201 ]; then
  echo "Failed 1: $status_code"
  exit 1
fi

#second put is ok
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 200 ]; then
  echo "Failed 2: $status_code"
  exit 1
fi

#get returns all
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" != '["photos"]' ]; then
  echo "Failed 3: $body"
  exit 1
fi

#delete returns 204
status_code=$(curl -X DELETE -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 204 ]; then
  echo "Failed 4: $status_code"
  exit 1
fi

#get returns none
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" != '[]' ]; then
  echo "Failed 5: $body"
  exit 1
fi

#bad blob name returns 400
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/pho_tos)
if [ "$status_code" -ne 400 ]; then
  echo "Failed 6: $status_code"
  exit 1
fi

#get returns none
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" != '[]' ]; then
  echo "Failed 7: $body"
  exit 1
fi

bash restart.sh

#get still returns none after restart
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" != '[]' ]; then
  echo "Failed 8: $body"
  exit 1
fi

#put now creates
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 201 ]; then
  echo "Failed 9: $status_code"
  exit 1
fi

bash restart.sh

#bucket survives restart
response=$(curl -s -w "\n%{http_code}" localhost:31415/buckets)
body=$(echo "$response" | sed '$d')
if [ "$body" != '["photos"]' ]; then
  echo "Failed 10: $body"
  exit 1
fi

echo "done."

bash down.sh