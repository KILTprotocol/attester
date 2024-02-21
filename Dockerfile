# Frontend Build Stage
FROM node:20.5.1 as frontend-build

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

# Build backend
RUN cargo build --release --bins

# Final Stage
FROM rust:slim-buster

ARG PORT=5656

WORKDIR /app

# Copy frontend build
COPY --from=frontend-build /usr/src/app/dist /usr/share/html

# Copy backend build
COPY --from=backend-build /app/target/release/attester_spiritnet /app/attester_spiritnet
COPY --from=backend-build /app/target/release/attester_peregrine /app/attester_peregrine

# Copy migrations config and scripts
COPY ./migrations /app/migrations
COPY ./scripts/start.sh /app/start.sh
VOLUME /app/config.yaml

EXPOSE ${PORT}

#start the application
CMD ["./start.sh" ]
