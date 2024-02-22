# Frontend Build Stage
FROM node:20.5.1 as frontend-build

ARG AUTH_URL=https://dev.opendid.kilt.io/api/v1/authorize
ARG BACKEND_URL=http://0.0.0.0:${port}/api/v1
ARG WSS_ENDPOINT=wss://peregrine.kilt.io:443/parachain-public-ws


ENV VITE_SIMPLE_REST_URL=${BACKEND_URL} \
    VITE_AUTH_URL=${AUTH_URL} \
    VITE_WSS_ENDPOINT=${WSS_ENDPOINT}

WORKDIR /usr/src/app

# Copy only package.json and yarn.lock first to leverage Docker cache
COPY ./frontend ./

# Install dependencies
RUN corepack enable && yarn set version stable && yarn

# Build frontend
RUN yarn build

# Backend Build Stage
FROM rust:buster as backend-build

ARG BUILD_FEATURE=--features=spiritnet

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install libpq-dev

WORKDIR /app

COPY . /app/


# Build backend
RUN cargo build --release --features=peregrine --bin=attester_peregrine
RUN cargo build --release --features=spiritnet --bin=attester_spiritnet

# Final Stage
FROM rust:slim-buster

ARG PORT=5656

WORKDIR /app

# Copy frontend build
COPY --from=frontend-build /usr/src/app/dist /usr/share/html

# Copy backend build
COPY --from=backend-build /app/target/release/attester-backend /app/attester-backend

# Copy migrations and config
COPY /migrations /app/migrations
VOLUME /app/config.yaml

EXPOSE ${PORT}

# Run migrations and start the application
CMD ["/app/attester-backend" , "/app/config.yaml"]
