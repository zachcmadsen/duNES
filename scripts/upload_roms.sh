#!/usr/bin/env bash

echo "Archiving and compressing roms..."
tar -czf roms.tar.gz roms

echo "Uploading roms.tar.gz..."
az storage blob upload \
    --account-name dunesroms \
    --container-name roms \
    --name roms.tar.gz \
    --file roms.tar.gz \
    --auth-mode login \
    --overwrite

echo "Removing roms.tar.gz..."
rm roms.tar.gz
