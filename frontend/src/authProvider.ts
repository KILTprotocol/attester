import { AuthProvider } from "react-admin";


export const authProvider: AuthProvider = {
  login: ({ token }) => {
  localStorage.setItem("token", token);
  window.location.href = "/"
  return Promise.resolve();
  },
  logout: () => {
    localStorage.removeItem("token");
    return Promise.resolve();
  },
  
  checkError: () => Promise.resolve(),
  
  checkAuth: () =>
    localStorage.getItem("token") ? Promise.resolve() : Promise.reject(),

  getPermissions: () => {
    return Promise.resolve(undefined);
  },

  getIdentity: () => {
    const persistedUser = localStorage.getItem("user");
    const user = persistedUser ? JSON.parse(persistedUser) : null;
    return Promise.resolve(user);
  },
};

export default authProvider;
