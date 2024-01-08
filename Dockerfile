FROM node:20.5.1 as frontend-build

WORKDIR /usr/src/app
COPY ./frontend/yarn.lock ./frontend/package.json ./frontend/.yarnrc.yml ./frontend/.yarn ./frontend/.env ./
RUN corepack enable && yarn set version stable && yarn install
COPY frontend ./

RUN yarn build

FROM rust:buster as backend-build

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install libpq-dev

WORKDIR /app
COPY . /app/

RUN cargo build --release --features spiritnet
RUN cargo install --root /app sqlx-cli

FROM rust:slim-buster

WORKDIR /app

COPY --from=frontend-build /usr/src/app/dist /usr/share/html
COPY --from=backend-build /app/target/release/attester-backend /app/attester-backend
COPY --from=backend-build /app/bin/sqlx /bin/sqlx
COPY /migrations /app/migrations
COPY /config.yaml /app

EXPOSE 7777

CMD [ "sh", "-c", "sqlx migrate run && /app/attester-backend /app/config.yaml" ]
