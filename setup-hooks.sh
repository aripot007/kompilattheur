#!/bin/sh

mkdir -p .git/hooks

cp .hooks/pre-commit .git/hooks/pre-commit

echo "Git hooks installed successfully!"