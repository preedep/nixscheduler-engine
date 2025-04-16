curl -v -X POST http://localhost:8888/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Task4",
    "cron": "*/5 * * * * * *",
    "task_type": "print",
    "payload": "{\"message\": \"Hello, World!\"}"
}'
