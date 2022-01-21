#/bin/bash

set -e

# Set all our secrets used in the worker
echo $SECRET_CF_API_TOKEN | wrangler secret put CF_API_TOKEN

# Start wrangler and register a trap to kill it once we're done
wrangler dev &
WRANGLER_PID=$!
trap "kill $WRANGLER_PID" EXIT


# Wait for port 8787 to open on wrangler by looping with netcat.
echo "\nWaiting wrangler to launch on 8787...\n"
while ! nc -z localhost 8787; do   
  sleep 0.1
done

# TODO
