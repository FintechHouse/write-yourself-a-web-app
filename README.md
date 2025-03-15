# Write Yourself a Web App in Rust

![Course banner](assets/banner.png)

## Introduction

Learning Rust can be challenging, but it's often easier with a concrete project in mind. This workshop is designed for people who want to learn Rust by building a practical, extensible API. We'll create a weather forecast API that fetches and caches data, handles user requests, and provides a simple frontend.

While building this little app, we'll cover many important Rust concepts such as:

- Async programming with Tokio
- Web frameworks (Axum)
- Error handling
- JSON serialization/deserialization with Serde
- Database interactions with SQLx
- HTTP client requests with Reqwest
- Template rendering
- Testing
- And more!

> [!NOTE]
> This course focuses on building a real-world API from scratch. We'll emphasize idiomatic Rust code and extensible design principles.

## Who's the Target Audience?

This workshop is intended for Rust beginners who are just starting their Rust journey. While prior Rust knowledge isn't necessary, familiarity with programming concepts is helpful.

We'll use popular crates from the Rust ecosystem, so you'll get hands-on experience with real-world Rust development practices.

### Necessary Tools

* [Rust](https://www.rust-lang.org/tools/install)
* [Git](https://git-scm.com/)
* [PostgreSQL](https://www.postgresql.org/download/) (or Docker for running PostgreSQL)

## Structure

Use `src/main.rs` as your starting point. If you get stuck, check out the [examples](/examples) folder, which contains working source code for each block. We recommend trying it yourself first and only referring to the example code if you encounter issues.

## Features We'll Cover

1. Setting up an Axum web server
2. Implementing API endpoints
3. Fetching and caching weather data
4. Database interactions with SQLx
5. Error handling and custom error types
6. Authentication middleware
7. HTML templating for a simple frontend
8. Testing our API

## Block 0 - Check Rust Installation

Run `rustc --version`.
You should see something like `rustc 1.68.0 (2c8cc3432 2023-03-06)`.

## Block 1 - Hello World API

* Set up a basic Axum server
* Create a "Hello, World!" endpoint
* Run the server and test with curl

## Block 2 - Weather API Integration

* Implement weather data fetching from an external API
* Create structs for API responses
* Handle JSON serialization/deserialization with Serde

## Block 3 - Database Integration

* Set up PostgreSQL database connection with SQLx
* Implement caching of weather data
* Create database queries for inserting and retrieving data

Some useful commands:

```bash
# Create a new database
docker run -d -p 5432:5432 -e POSTGRES_USER=forecast -e POSTGRES_PASSWORD=forecast -e POSTGRES_DB=forecast -d postgres
```

```bash
# Set up the database URL
export DATABASE_URL="postgres://forecast:forecast@localhost:5432/forecast?sslmode=disable" 
```

```bash
# Check DB contents
docker exec -it <container_id> psql -U forecast forecast
```

```sql
SELECT * FROM cities;
```

## Block 4 - Error Handling

* Create custom error types
* Implement error handling for API requests and database operations
* Use the `?` operator for concise error propagation

## Block 5 - Authentication Middleware

* Implement basic authentication middleware
* Protect certain routes with authentication. For example, the `/stats` endpoint, which shows the list of cached cities.
* Handle auth errors and responses

After this, access to the `stats` endpoint should be protected. You can test it with:

```bash
curl -u forecast:forecast http://localhost:3000/stats
```

and by passing the wrong credentials.


## Block 6 - Frontend Integration

* Add HTML templating with Askama
* Create a simple frontend for weather queries
* Integrate the frontend with the API

## Block 7 - Testing

* Write unit tests for API endpoints
* Implement integration tests
* Use test utilities provided by Axum and SQLx

## Choose Your Own Adventure

* Implement additional weather API features
* Add user registration and JWT authentication
* Create a CLI client for your API
* Optimize performance with caching strategies
* Deploy your API to a cloud provider

It's your choice! We're here to help you expand on the base project.

## Show And Tell!

We're excited to see what you build! If you'd like to share your Rust API project with us, please send us a link to your repository. We'll add it to the list below.

We'd love to hear your thoughts on the following:

- What was your biggest learning from this project?
- Which parts were most challenging?
- What aspects of Rust did you find most useful for API development?
- How would you extend this project further?
- What other Rust topics would you like to explore next?

## Closing Words

If you enjoyed this workshop, please share it with your friends and colleagues. It would be a great help if you could tweet about it or share it on [Reddit](https://www.reddit.com/r/rust/) or [LinkedIn](https://www.linkedin.com/).

You might also want to [subscribe to our newsletter](https://example.com/newsletter) for future workshops and other Rust content.

If you're looking for professional Rust training, please get in touch with us at [example.com](https://example.com/).
