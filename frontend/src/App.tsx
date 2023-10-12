import { Admin, Resource } from "react-admin";

import { Layout, Login } from "./layout";
import { darkTheme, lightTheme } from "./layout/themes";
import { dataProvider } from "./dataProvider";
import { authProvider } from "./authProvider";
import { Dashboard } from "./Dashboard";
import {
  AttestationEdit,
  AttestationList,
  AttestationCreate
} from "./attestation_request";

export const App = () => (
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
    <Resource
      name="attestation_request"
      list={AttestationList}
      edit={AttestationEdit}
      create={AttestationCreate}
    />
  </Admin>
);
