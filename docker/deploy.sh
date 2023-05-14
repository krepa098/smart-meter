#!/bin/bash

host=$1
remote_dir="/share/CACHEDEV1_DATA"
local_image="./thrsensor.tgz"

echo -n "Username: "
read user

sftp $user@$host:$remote_dir <<< "put $local_image"

if [ $? -eq 0 ]; then
    echo "Finished uploading image"
    read -p "Press enter to continue"
    echo "Executing docker load..."
    ssh $user@$host "bash -c \"source /etc/profile && gunzip -c $remote_dir/thrsensor.tgz | docker load && rm $remote_dir/thrsensor.tgz\""
    echo "done"
else
    echo "Cannot upload image"
fi
