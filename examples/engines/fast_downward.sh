#!/bin/bash

DOMAIN="$1"
PROBLEM="$2"
PLAN_OUT="$3"

fast-downward.py --plan-file ${PLAN_OUT} --alias lama ${DOMAIN} ${PROBLEM}
