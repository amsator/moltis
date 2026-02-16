#!/usr/bin/env bash
cd /home/m/Documents/moltis-wt
nix-shell --run "./target/release/moltis gateway"
