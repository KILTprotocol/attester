import { Admin, Resource } from 'react-admin'

import { Layout, Login } from './layout'
import { darkTheme, lightTheme } from './layout/themes'
import { dataProvider } from './api/dataProvider'
import { authProvider } from './api/authProvider'
import Dashboard from './components/Dashboard'
import { AttestationCreate } from './components/AttestationAdd'
import { AttestationEdit } from './components/AttestationEdit'
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
      name='attestationRequest'
      list={AttestationList}
      edit={AttestationEdit}
      create={AttestationCreate}
    />
  </Admin>
)
