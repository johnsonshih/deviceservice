CONTAINER_REPOSITORY=ghcr.io/johnsonshih/akri
docker login $CONTAINER_REPOSITORY
docker build -t ghcr.io/johnsonshih/nginx-mtls:latest -f Dockerfile .
docker push ghcr.io/johnsonshih/nginx-mtls:latest

