curl -v -X POST http://localhost:8888/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Task-ADF",
    "cron": "0 */5 * * * *",
    "task_type": "adf_pipeline",
    "payload": "{ \"subscription_id\": \"9d3ff024-cfad-4108-a098-8e675fbc4cc4\", \"resource_group\": \"RG-SG-NICKDEV001\", \"factory_name\": \"MyNICKADF001\", \"pipeline\": \"batch_account_java\", \"parameters\": { \"message\": \"Hello, World!\" } }"
}'