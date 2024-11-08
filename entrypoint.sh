#!/bin/bash
#

if [[ -z "${UID}" ]]; then
  echo "UID is not set. Using default UID 1005."
  UID="1005"
fi
if [[ -z "${TZ}" ]]; then
  echo "TZ is not set. Using default timezone 'UTC'."
  TZ="UTC"
fi
cp "/usr/share/zoneinfo/${TZ}" /etc/localtime 
echo "${TZ}" > /etc/timezone
groupadd -g 1005 homers
useradd -u 1005 -g 1005 -s /bin/bash -m homers

runuser --user homers -- "$@" 
