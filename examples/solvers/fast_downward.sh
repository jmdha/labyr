#!/bin/bash

OUT="$1"
ALIAS="$2"
DOMAIN="$3"
PROBLEM="$4"

shopt -s expand_aliases
source ~/.bash_aliases 

# Assumes that fast-downward.py is aliased as downward
downward --plan-file ${OUT} --alias ${ALIAS} ${DOMAIN} ${PROBLEM}
