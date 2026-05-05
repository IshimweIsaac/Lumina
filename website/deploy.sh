#!/bin/bash
# Phase 0: Chapter 41 WASM Deployment Script
# Copies the latest wasm-pack build into the website public directory.

SOURCE_DIR="../crates/lumina-wasm/pkg"
TARGET_DIR="./public/lumina-wasm"

echo "Deploying Lumina WASM to website assets..."

if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: WASM build not found at $SOURCE_DIR"
    exit 1
fi

mkdir -p "$TARGET_DIR"
cp "$SOURCE_DIR"/*.wasm "$TARGET_DIR/"
cp "$SOURCE_DIR"/*.js "$TARGET_DIR/"

echo "Successfully deployed WASM assets."
