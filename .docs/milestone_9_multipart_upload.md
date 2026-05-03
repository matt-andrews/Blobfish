# Milestone 9: Multipart Upload

**Goal:** Allow large objects to be uploaded in separately committed parts — each part independently retryable. Parts can arrive out of order; the object is only visible after `CompleteMultipartUpload`.

## Deliverables

- [ ] `POST /objects/{bucket}/{*key}?uploads` → initiate, returns `upload_id`
- [ ] `PUT /objects/{bucket}/{*key}?uploadId={id}&partNumber={n}` → upload part, returns `ETag` for part
- [ ] `GET /objects/{bucket}/{*key}?uploadId={id}` → list uploaded parts
- [ ] `POST /objects/{bucket}/{*key}?uploadId={id}` (with part list body) → complete upload
- [ ] `DELETE /objects/{bucket}/{*key}?uploadId={id}` → abort upload, cleans up part chunks
- [ ] Parts numbered 1–10000 (S3 convention)
- [ ] Each part is at least 5 MiB except the last (optional enforcement for MVP)
- [ ] Completed object is assembled from parts in part-number order, not arrival order
- [ ] Part ETags in `CompleteMultipartUpload` request body are validated against stored part ETags
- [ ] Stale incomplete uploads cleaned up at startup (already handled by `staging_ttl_seconds`)
- [ ] Multipart upload sessions tracked in redb `uploads` table

## API

**Initiate:**
```
POST /objects/{bucket}/{*key}?uploads
Response 200: { "upload_id": "..." }
```

**Upload Part:**
```
PUT /objects/{bucket}/{*key}?uploadId={id}&partNumber={n}
Body: part bytes
Response 200:
  ETag: "sha256hex_of_this_part"
```

**List Parts:**
```
GET /objects/{bucket}/{*key}?uploadId={id}
Response 200: { "upload_id": "...", "parts": [{ "part_number": 1, "size_bytes": ..., "etag": "..." }] }
```

**Complete:**
```
POST /objects/{bucket}/{*key}?uploadId={id}
Body: { "parts": [{ "part_number": 1, "etag": "..." }, ...] }
Response 201: version headers (same as single-PUT response)
```

**Abort:**
```
DELETE /objects/{bucket}/{*key}?uploadId={id}
Response 204
```

## Storage Model

Each part is stored as one or more chunks (same chunking rules as single PUT). Parts reference their chunks in an `UploadPart` record in the `uploads` table.

On `CompleteMultipartUpload`:
1. Validate all part ETags match stored values.
2. Collect all chunk descriptors from all parts in part-number order.
3. Build a single `ObjectManifest` covering all parts' chunks.
4. Commit `ObjectVersion` + `ObjectManifest` atomically.
5. Delete the upload session record.

On `AbortMultipartUpload`:
1. Delete all chunk files associated with the upload.
2. Delete the upload session record.

## Learning Focus

- Upload session state machine (initiated → parts arriving → completed/aborted)
- Out-of-order part handling (store all, assemble at complete time)
- Idempotent part uploads (same part number + same bytes = same ETag, re-upload safe)
- Cleaning up partial state on abort

## Done When

```bash
# Split a 100 MiB file into 10 MiB parts and upload
UPLOAD_ID=$(curl -s -X POST localhost:8080/objects/photos/big.bin?uploads | jq -r .upload_id)

split -b 10M big.bin part_
PART=1
for f in part_*; do
  curl -X PUT --data-binary @$f \
    "localhost:8080/objects/photos/big.bin?uploadId=$UPLOAD_ID&partNumber=$PART"
  PART=$((PART+1))
done

curl -X POST "localhost:8080/objects/photos/big.bin?uploadId=$UPLOAD_ID" \
  -H "Content-Type: application/json" \
  -d '{"parts":[{"part_number":1,"etag":"..."},...]}'

curl localhost:8080/objects/photos/big.bin -o result.bin
diff big.bin result.bin   # identical
```
