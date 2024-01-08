import { Admin, Resource } from 'react-admin'

import { Layout, Login } from './layout'
import { darkTheme, lightTheme } from './layout/themes'
import { dataProvider } from './api/dataProvider'
import { authProvider } from './api/authProvider'
import Dashboard from './components/Dashboard'
import { AttestationCreate } from './components/AttestationAdd'
import { AttestationList } from './components/AttestationList'

export const App = () => (
  <Admin
    dataProvider={dataProvider}
    authProvider={authProvider}
    loginPage={Login}
    layout={Layout}
    theme={lightTheme}
    darkTheme={darkTheme}
    defaultTheme='light'
    dashboard={Dashboard}
  >
    <Resource
      name='attestation_request'
      list={AttestationList}
      create={AttestationCreate}
    />
  </Admin>
)
