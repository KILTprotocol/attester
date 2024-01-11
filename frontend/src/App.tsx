import { Admin, Resource } from 'react-admin'

import { Layout, Login } from './layout'
import { darkTheme, lightTheme } from './layout/themes'
import { dataProvider } from './api/dataProvider'
import { authProvider } from './api/authProvider'
import { AttestationCreate, AttestationList, Dashboard } from './components'

export function App() {
  return (
    <Admin
      dataProvider={dataProvider}
      authProvider={authProvider}
      loginPage={Login}
      layout={Layout}
      theme={lightTheme}
      darkTheme={darkTheme}
      defaultTheme="light"
      dashboard={Dashboard}
    >
      <Resource name="attestation_request" list={AttestationList} create={AttestationCreate} />
    </Admin>
  )
}
