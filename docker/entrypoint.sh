#!/bin/sh

set -e

# Set default values for user creation if not provided
: ${INIT_USER_NAME:=super}
: ${INIT_USER_PASSWORD:=dev}
: ${INIT_USER_EMAIL:=super@example.com}

# Run the mqttpal create user command with provided environment variables
mqttpal create-init-user "$INIT_USER_NAME" "$INIT_USER_PASSWORD" "$INIT_USER_EMAIL"

# Start the mqttpal service
exec mqttpal serve
