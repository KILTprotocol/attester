FROM node:20.5.1 as frontend-build

WORKDIR /usr/src/app
COPY ./frontend/yarn.lock ./frontend/package.json ./
RUN yarn 
COPY frontend ./

RUN yarn build

FROM rust:slim-buster as backend-build

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install libpq-dev

WORKDIR /app
COPY . /app/

RUN cargo build --release --features spiritnet


FROM rust:slim-buster

COPY --from=frontend-build /usr/src/app/dist /usr/share/nginx/html
COPY --from=backend-build /app/target/release/attester-backend /app/attester-backend

EXPOSE 7777

CMD [ "cargo sqlx prepare && ./app/attester-backend" ]



