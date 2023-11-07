import { AuthProvider } from "react-admin";
import jwtDecode from "jwt-decode";

interface JWTPayload {
  aud: string;
  exp: number;
  iat: number;
  iss: string;
  nonce: string;
  pro: { [key: string]: any };
  sub: string;
  w3n: string;
}

export const authProvider: AuthProvider = {
  login: (token) => {
    try {
      let decodedToken = jwtDecode<JWTPayload>(token);
      if (Object.keys(decodedToken.pro).length) {
        localStorage.setItem("role", "admin");
      } else {
        localStorage.setItem("role", "user");
      }
      localStorage.setItem("token", token);
      window.location.href = "/";
      return Promise.resolve();
    } catch (Error) {
      return Promise.reject();
    }
  },

  logout: () => {
    localStorage.removeItem("token");
    localStorage.removeItem("role");
    return Promise.resolve();
  },

  checkError: (error) => {
    if (!error) {
      return Promise.resolve();
    }
    const status = error.status;
    if (status === 401 || status === 403) {
      localStorage.removeItem("token");
      localStorage.removeItem("role");
      return Promise.reject();
    }
    return Promise.resolve();
  },

  checkAuth: () =>
    localStorage.getItem("token") ? Promise.resolve() : Promise.reject(),

  getRole: () => {
    return localStorage.getItem("role");
  },

  getPermissions: () => Promise.reject(undefined),

  getToken: () => localStorage.getItem("token"),
};

export default authProvider;
