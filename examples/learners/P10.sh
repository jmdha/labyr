#!/bin/bash

OUT="$1"
GENERATION_STRATEGY="$2"
DOMAIN="$3"
PROBLEMS="$4"

shopt -s expand_aliases
source ~/.bash_aliases 

# Assumes that P10 exe is aliased as P10Train (temp name until something better is defined)
# Assumes that modified stackelberg is aliased as stackelberg
P10Train --domain ${DOMAIN} --problems ${PROBLEMS} --generation-strategies ${GENERATION_STRATEGY} --stackelberg-path $(stackelberg)
