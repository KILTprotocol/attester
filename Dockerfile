# Frontend Build Stage
FROM node:20.5.1 as frontend-build

ARG build_features=--no-default-features
ARG port=5656
ARG backend_url=http://localhost:5656/api/v1
ARG auth_url=http://localhost:4444/api/v1/authorize
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

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install libpq-dev

WORKDIR /app

COPY . /app/

# Install sqlx-cli
RUN cargo install --root /app sqlx-cli

# Build backend
RUN cargo build --release ${build_features}

# Final Stage
FROM rust:slim-buster

WORKDIR /app

# Copy frontend build
COPY --from=frontend-build /usr/src/app/dist /usr/share/html

# Copy backend build
COPY --from=backend-build /app/target/release/attester-backend /app/attester-backend
COPY --from=backend-build /app/bin/sqlx /bin/sqlx

# Copy migrations and config
COPY /migrations /app/migrations
COPY /config.yaml /app

EXPOSE ${port}

# Run migrations and start the application
CMD ["sh", "-c", "sqlx migrate run && /app/attester-backend /app/config.yaml"]