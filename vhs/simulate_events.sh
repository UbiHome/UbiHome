#!/bin/sh
# Simulates a short motion event in the background while `ubihome run` is recording.
(
  sleep 5
  echo "true" > motion
  sleep 1
  echo "false" > motion
) > /dev/null 2>&1 &
