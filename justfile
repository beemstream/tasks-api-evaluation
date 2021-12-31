test:
    sh .env
    mongo --eval "db.getCollection('task').remove({})" $DATABASE_URL
    cargo test -- --test-threads=1 --nocapture
perf:
    sh .env
    cargo run --release -- --host $API_ENDPOINT --run-time 2m --status-codes
    mongo --eval "db.getCollection('task').remove({})" $DATABASE_URL
