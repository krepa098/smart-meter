#!/bin/bash

export DOCKER_BUILDKIT=1

SSH_KEY=$HOME/.ssh/smart-meter-deploy
if sudo docker build --no-cache --ssh default=$SSH_KEY -t thrsensor . ; then
    echo "Saving image..."
    IMAGE_FILE=thrsensor.tgz
    docker save thrsensor | gzip > $IMAGE_FILE

    echo "Use 'gunzip -c $IMAGE_FILE | docker load' to restore"
else
    echo "docker build failed!"
fi
