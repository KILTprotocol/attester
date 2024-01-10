# Frontend Build Stage
FROM node:20.5.1 as frontend-build

ARG auth_url=https://dev.opendid.kilt.io/api/v1/authorize
ARG backend_url=http://0.0.0.0:${port}/api/v1
ARG wss_endpoint=wss://peregrine.kilt.io:443/parachain-public-ws


ENV VITE_SIMPLE_REST_URL=${backend_url} \
    VITE_AUTH_URL=${auth_url} \
    VITE_WSS_ENDPOINT=${wss_endpoint}

WORKDIR /usr/src/app

# Copy only package.json and yarn.lock first to leverage Docker cache
COPY ./frontend ./

# Install dependencies
RUN corepack enable && yarn set version stable && yarn

# Build frontend
RUN yarn build

# Backend Build Stage
FROM rust:buster as backend-build

ARG args=--features=spiritnet

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install libpq-dev

WORKDIR /app

COPY . /app/


# Build backend
RUN cargo build --release --bin=attester-backend --package=attester-backend $args

# Final Stage
FROM rust:slim-buster

ARG port=5656

WORKDIR /app

# Copy frontend build
COPY --from=frontend-build /usr/src/app/dist /usr/share/html

# Copy backend build
COPY --from=backend-build /app/target/release/attester-backend /app/attester-backend

# Copy migrations and config
COPY /migrations /app/migrations
VOLUME /app/config.yaml

EXPOSE ${port}

# Run migrations and start the application
CMD ["/app/attester-backend" , "/app/config.yaml"]
