#!/bin/sh

docker build -t chrome .
docker tag chrome:latest 608440221714.dkr.ecr.us-east-2.amazonaws.com/chrome:latest
docker push 608440221714.dkr.ecr.us-east-2.amazonaws.com/chrome:latest