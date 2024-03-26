#!/bin/bash

OUT="$1"
LEARNER_DIRECTORY="$2"
ALIAS="$3"
DOMAIN="$4"
PROBLEM="$5"

shopt -s expand_aliases
source ~/.bash_aliases 

# Assumes that fast-downward.py is aliased as downward
downward --plan-file ${OUT} --alias ${ALIAS} ${LEARNER_DIRECTORY}/output/enhancedDomain.pddl ${PROBLEM}
