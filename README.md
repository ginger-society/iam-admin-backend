# SampleService

implemented in Rust using Rocket.rs, integrating MongoDB and PostgreSQL with the Diesel ORM.


## 🔧 Building and Testing

### Debug Mode

```bash
cargo run
```

### Release Mode

```bash
cargo build --release && cargo run --release
```

### Unit Testing

```bash
cargo test
```

In the GH actions , you need to have the following secrets : 

DOCKER_HUB_ACCESS_TOKEN
DOCKER_HUB_USERNAME
GH_TOKEN
GINGER_TOKEN
STAGING_K8_CONFIG
SERVICE_IMAGE_PREFIX