import { useCallback, useEffect } from "react";
import {
  Avatar,
  Button,
  Card,
  CardActions,
} from "@mui/material";
import LockIcon from "@mui/icons-material/Lock";
import { Form } from "react-admin";

import Box from "@mui/material/Box";
import authProvider from "../authProvider";

const Login = () => {

  const handleSubmit = useCallback(() => {
    let clientId;
    const nonce = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    const state = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    if (window.location.href.includes("client_id")) {
      clientId = window.location.hash.slice(8).split("=")[1]
    }
    let url = new URL(import.meta.env.VITE_AUTH_URL);
    url.searchParams.append("response_type", "id_token");
    url.searchParams.append("client_id", clientId ? clientId : import.meta.env.VITE_CLIENT_ID);
    url.searchParams.append("redirect_uri", window.location.origin + "/#/login");
    url.searchParams.append("scope", "openid");
    url.searchParams.append("state", state);
    url.searchParams.append("nonce", nonce);
    window.location.href = url.toString();
  }, []);

  useEffect(() => {
    const params = new URLSearchParams(window.location.hash.slice(8));
    const token = params.get("id_token");
    if (token) {
      authProvider.login(token)
    }
  }, [])

  return (
    <Form onSubmit={handleSubmit} noValidate>

      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          minHeight: "100vh",
          alignItems: "center",
          justifyContent: "flex-start",
          background: "url(https://source.unsplash.com/featured/1600x1600)",
          backgroundRepeat: "no-repeat",
          backgroundSize: "cover"
        }}
      >
        <Card sx={{ minWidth: 300, marginTop: "6em" }}>
          <Box
            sx={{
              margin: "1em",
              display: "flex",
              justifyContent: "center"
            }}
          >
            <Avatar sx={{ bgcolor: "secondary.main" }}>
              <LockIcon />
            </Avatar>
          </Box>
          <Box
            sx={{
              marginTop: "1em",
              display: "flex",
              justifyContent: "center",
              color: (theme) => theme.palette.grey[500]
            }}
          >
          </Box>
          <CardActions sx={{ padding: "0 1em 1em 1em" }}>
            <Button
              variant="contained"
              type="submit"
              color="primary"
              fullWidth
            >
              Login with KILT
            </Button>
          </CardActions>
        </Card>
      </Box>
    </Form>
  );
};


export default Login;
