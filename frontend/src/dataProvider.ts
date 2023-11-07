import simpleRestProvider from "ra-data-simple-rest";
import authProvider from "./authProvider";
import { fetchUtils } from "react-admin";
import axios from "axios";

export const getAxiosClient = async () => {
  const token = await authProvider.getToken();
  let instance = axios.create({
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  return instance;
};

const httpClient = async (
  url: string,
  options: { [key: string]: any } = {}
) => {
  const token = await authProvider.getToken();
  if (!options.headers) {
    options.headers = new Headers({ Accept: "application/json" });
  }
  options.user = {
    authenticated: true,
    token: `Bearer ${token}`,
  };
  return fetchUtils.fetchJson(url, options);
};

export const dataProvider = simpleRestProvider(
  import.meta.env.VITE_SIMPLE_REST_URL,
  httpClient
);
