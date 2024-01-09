docker build -t deviceservice:v2 -f Dockerfile .

CONTAINER_REPOSITORY=ghcr.io/johnsonshih/akri
docker login $CONTAINER_REPOSITORY
docker build -t ghcr.io/johnsonshih/deviceservice:latest -f Dockerfile .
docker push ghcr.io/johnsonshih/deviceservice:latest

