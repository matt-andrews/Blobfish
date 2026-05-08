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

# delete any trailing data that could break the tests
curl -X POST localhost:31415/detachz

#
# create bucket
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 201 ]; then
  echo "Failed 1: $status_code"
  exit 1
fi

echo "putting image";
# put the image
curl -X PUT --data-binary @cat.jpg -H "content-type: image/jpeg" localhost:31415/objects/photos/cat.jpg

# get file and validate
OUT="out.jpg"
trap 'rm -f "$OUT"' EXIT

echo "getting image";
curl localhost:31415/objects/photos/cat.jpg -f -o "$OUT"

if ! diff -q "cat.jpg" "$OUT" > /dev/null; then
    echo "Files do not match!" >&2
    exit 1
fi

echo "Files match!"

#test headers
HEADERS=$(curl -sI -X GET localhost:31415/objects/photos/cat.jpg)

if ! echo "$HEADERS" | grep -q "content-type: image/jpeg"; then
    echo "Missing or wrong Content-Type" >&2
    exit 1
fi

if ! echo "$HEADERS" | grep -q "content-length: 1155778"; then
    echo "Missing or wrong Content-Length" >&2
    exit 1
fi

if ! echo "$HEADERS" | grep -q 'etag: "39803b7116614c984b763b8a35a8c751c9816813d2737742230fa4be1e5d7ac1-1"'; then
    echo "Missing ETag" >&2
    exit 1
fi

if ! echo "$HEADERS" | grep -q "x-blobfish-sha256: 1c9cd40b037bbac4f4b236de657bab130200bb037c5df01372cc72b10144d7ab"; then
    echo "Missing X-Blobfish-Checksum-Sha256" >&2
    exit 1
fi

echo "Headers Match"

# delete img
status_code=$(curl -X DELETE -s -o /dev/null -w "%{http_code}" localhost:31415/objects/photos/cat.jpg)
if [ "$status_code" -ne 204 ]; then
  echo "Failed 3: $status_code"
  exit 1
fi

# validate 404 after deleted image
status_code=$(curl -s -o /dev/null -w "%{http_code}" localhost:31415/objects/photos/cat.jpg)
if [ "$status_code" -ne 404 ]; then
  echo "Failed 4: $status_code"
  exit 1
fi

bash restart.sh

# validate still 404 after restart
status_code=$(curl -s -o /dev/null -w "%{http_code}" localhost:31415/objects/photos/cat.jpg)
if [ "$status_code" -ne 404 ]; then
  echo "Failed 4: $status_code"
  exit 1
fi
#
#

# validate integrity check
status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" --data-binary @cat.jpg -H "content-type: image/jpeg" -H "x-blobfish-sha256: 1c9cd40b037bbac4f4b236de657bab130200bb037c5df01372cc72b10144d7ab-WRONG" localhost:31415/objects/photos/cat.jpg)
if [ "$status_code" -ne 400 ]; then
  echo "Failed 5: $status_code"
  exit 1
fi

status_code=$(curl -X PUT -s -o /dev/null -w "%{http_code}" --data-binary @cat.jpg -H "content-type: image/jpeg" -H "x-blobfish-sha256: 1c9cd40b037bbac4f4b236de657bab130200bb037c5df01372cc72b10144d7ab" localhost:31415/objects/photos/cat.jpg)
if [ "$status_code" -ne 200 ]; then
  echo "Failed 6: $status_code"
  exit 1
fi

# delete bucket
status_code=$(curl -X DELETE -s -o /dev/null -w "%{http_code}" localhost:31415/buckets/photos)
if [ "$status_code" -ne 204 ]; then
  echo "Failed 7: $status_code"
  exit 1
fi

echo "done."

bash down.sh