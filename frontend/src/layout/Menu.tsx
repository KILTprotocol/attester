import Box from "@mui/material/Box"
import AppsIcon from "@mui/icons-material/Apps"
import { DashboardMenuItem, useSidebarState, MenuItemLink } from "react-admin"

const Menu = () => {
  const [open] = useSidebarState()

  return (
    <Box
      sx={{
        width: open ? 200 : 50,
        marginTop: 1,
        marginBottom: 1,
        transition: (theme) =>
          theme.transitions.create("width", {
            easing: theme.transitions.easing.sharp,
            duration: theme.transitions.duration.leavingScreen,
          }),
      }}
    >
      <DashboardMenuItem />
      <MenuItemLink
        to="attestation_request"
        state={{ _scrollToTop: true }}
        primaryText={"Attestations"}
        leftIcon={<AppsIcon />}
      />
    </Box>
  )
}

export default Menu
