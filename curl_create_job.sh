curl -v -X POST http://localhost:8888/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "id": "job-001",
    "name": "Say Hello",
    "cron": "*/5 * * * * * *",
    "task_type": "print",
    "payload": "Hello from Rust!"
}'
