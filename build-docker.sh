# สร้าง image
docker build -t nixscheduler-engine .

# Run container
#docker run -it --rm -p 8888:8888 --env-file .env nixscheduler-engine