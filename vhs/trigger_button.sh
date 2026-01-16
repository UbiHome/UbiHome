#!/bin/sh
sleep 5 && curl -s -X POST http://localhost:8080/button/mysensor/press & >/dev/null 2>/dev/null