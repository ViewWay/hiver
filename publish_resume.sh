#!/bin/bash
# Resume publishing remaining crates
set -e

CRATES=(
  # Level 0: No unpublished hiver deps
  hiver-cloud
  hiver-data-macros
  hiver-exceptions
  hiver-extractors
  hiver-graphql
  hiver-hateoas
  hiver-macros
  hiver-micrometer
  hiver-middleware
  hiver-multipart
  hiver-response
  hiver-resilience
  hiver-retry
  hiver-router
  hiver-schedule
  hiver-security
  hiver-shell
  hiver-state-machine
  hiver-test
  hiver-tx
  hiver-validation
  hiver-vault
  hiver-websocket-stomp

  # Level 1: Depends on level 0 or already-published crates
  hiver-data-annotations
  hiver-data-redis
  hiver-data-mongodb
  hiver-observability
  hiver-openapi

  # Level 2: Depends on level 0-1
  hiver-cache
  hiver-data-rdbc

  # Level 3: Depends on level 0-2
  hiver-session
  hiver-data-orm

  # Level 4: Meta-crate, all deps optional
  hiver-starter
)

WAIT=660

echo "=== Resuming publish: ${#CRATES[@]} crates to go ==="

for crate in "${CRATES[@]}"; do
  TIMESTAMP=$(date -u '+[%H:%M:%S UTC]')
  echo "$TIMESTAMP PUBLISHING: $crate"

  cd /Users/yimiliya/RustroverProjects/hiver/crates/$crate

  STATUS=$(cargo publish --allow-dirty 2>&1) || true
  if echo "$STATUS" | grep -q "is already uploaded"; then
    echo "$TIMESTAMP SKIP (already published): $crate"
  elif echo "$STATUS" | grep -q "Published"; then
    echo "$TIMESTAMP SUCCESS: $crate"
  else
    echo "$TIMESTAMP OUTPUT:"
    echo "$STATUS"
    if echo "$STATUS" | grep -qE "error|failed"; then
      echo "$TIMESTAMP FAILED: $crate - stopping"
      exit 1
    fi
  fi

  if [ "$crate" != "${CRATES[${#CRATES[@]}-1]}" ]; then
    echo "$TIMESTAMP Waiting ${WAIT}s before next crate..."
    sleep $WAIT
  fi
done

echo "=== All done! ==="
