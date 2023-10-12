FROM node:20.5.1 as build-deps

WORKDIR /usr/src/app
COPY yarn.lock package.json ./
RUN yarn 
COPY . ./

RUN yarn build



FROM nginx:1.12-alpine
COPY --from=build-deps /usr/src/app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]