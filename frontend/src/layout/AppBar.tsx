import { AppBar, TitlePortal } from 'react-admin'
import { Box, useMediaQuery, Theme } from '@mui/material'

export default function CustomAppBar() {
  const isLargeEnough = useMediaQuery<Theme>((theme) => theme.breakpoints.up('sm'))
  return (
    <AppBar color="secondary" elevation={1}>
      <TitlePortal />

      {isLargeEnough && <Box component="span" sx={{ flex: 1 }} />}
    </AppBar>
  )
}
