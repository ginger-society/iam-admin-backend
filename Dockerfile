FROM gingersociety/rust-rocket-api-builder:latest as builder

ARG GINGER_TOKEN

RUN bash -c "$(curl -fsSL https://raw.githubusercontent.com/ginger-society/infra-as-code-repo/main/rust-helpers/installer.sh)" -- ginger-society/ginger-db:latest

# Create a new directory for the app
WORKDIR /app
COPY . .
# Run the ginger-auth command and capture the output
RUN ginger-auth token-login $GINGER_TOKEN
RUN ginger-connector connect stage-k8
RUN ginger-db render --skip
# Build the application in release mode
RUN cargo build --release

# Second stage: Create the minimal runtime image
FROM gingersociety/rust-rocket-api-runner:latest

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/IAMAdminService /app/

# Set the working directory
WORKDIR /app

# Run the executable when the container starts
ENTRYPOINT ["./IAMAdminService"]