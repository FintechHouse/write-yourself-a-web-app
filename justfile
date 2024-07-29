# test
test:
    curl "http://localhost:3000/weather?city=London"
    
# run database
db:
    docker run -d -p 5432:5432 -e POSTGRES_USER=forecast -e POSTGRES_PASSWORD=forecast -e POSTGRES_DB=forecast -d postgres