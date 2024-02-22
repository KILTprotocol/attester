import { useCallback, useEffect } from 'react'
import { Avatar, Button, Card, CardActions } from '@mui/material'
import LockIcon from '@mui/icons-material/Lock'
import { Utils } from '@kiltprotocol/sdk-js'
import Box from '@mui/material/Box'
import authProvider from '../api/authProvider'
import { getAuthorizeUrl } from '../utils/utils'

export default function Login() {
  const handleSubmit = useCallback((clientId: string) => {
    const nonce = Utils.UUID.generate()
    const state = Utils.UUID.generate()

    const url = new URL(getAuthorizeUrl())
    url.searchParams.append('response_type', 'id_token')
    url.searchParams.append('client_id', clientId as string)
    url.searchParams.append('redirect_uri', `${window.location.origin}/#/login`)
    url.searchParams.append('scope', 'openid')
    url.searchParams.append('state', state)
    url.searchParams.append('nonce', nonce)
    window.location.href = url.toString()
  }, [])

  useEffect(() => {
    const params = new URLSearchParams(window.location.hash.slice(8))
    const token = params.get('id_token')
    if (token) {
      authProvider.login(token)
    }
  }, [])

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        minHeight: '100vh',
        alignItems: 'center',
        justifyContent: 'flex-start',
        background: 'url(https://source.unsplash.com/featured/1600x1600)',
        backgroundRepeat: 'no-repeat',
        backgroundSize: 'cover',
      }}
    >
      <Card sx={{ minWidth: 300, marginTop: '6em' }}>
        <Box
          sx={{
            margin: '1em',
            display: 'flex',
            justifyContent: 'center',
          }}
        >
          <Avatar sx={{ bgcolor: 'secondary.main' }}>
            <LockIcon />
          </Avatar>
        </Box>
        <Box
          sx={{
            marginTop: '1em',
            display: 'flex',
            justifyContent: 'center',
            color: (theme) => theme.palette.grey[500],
          }}
        />
        <CardActions sx={{ padding: '0 1em 1em 1em' }}>
          <div style={{ flexDirection: 'column' }}>
            <Button
              variant="contained"
              type="submit"
              sx={{ marginBottom: '1em' }}
              color="primary"
              onClick={() => handleSubmit('example-client')}
              fullWidth
            >
              Login as Employee
            </Button>
            <Button
              variant="contained"
              type="submit"
              color="secondary"
              fullWidth
              onClick={() => handleSubmit('default')}
            >
              Login as User
            </Button>
          </div>
        </CardActions>
      </Card>
    </div>
  )
}
