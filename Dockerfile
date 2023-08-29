FROM rust:1.69.0

# Create app directory
WORKDIR /usr/src/app

RUN cargo init

# Install app dependencies
# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

RUN cargo fetch

# Bundle app source
COPY . .

# Document port intended to be published
EXPOSE 5000

# Set the default command to run your application
CMD ["cargo", "run"]
