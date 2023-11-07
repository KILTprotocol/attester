import {
  List,
  Datagrid,
  TextField,
  DateField,
  useRecordContext,
  EditButton,
  useNotify,
  useTheme,
  useRefresh,
} from "react-admin";
import ReactJson from "react-json-view";
import Fab from "@mui/material/Fab";
import DoneIcon from "@mui/icons-material/Done";
import RemoveIcon from "@mui/icons-material/Remove";
import CircularProgress from "@mui/material/CircularProgress";
import Tooltip from "@mui/material/Tooltip";

import { AttestationRequsts } from "../types";
import { useState } from "react";
import { isUserAdmin } from "../utils";
import { getAxiosClient } from "../dataProvider";
import { ICType } from "@kiltprotocol/sdk-js";

const ExpandAttestation = () => {
  const record = useRecordContext<AttestationRequsts>();
  const [theme, _] = useTheme();
  return (
    <ReactJson
      theme={theme === "dark" ? "colors" : "bright:inverted"}
      src={record.credential.claim.contents}
    />
  );
};

const ApproveButton = () => {
  const record = useRecordContext<AttestationRequsts>();
  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
  const [isLoading, setIsLoading] = useState(false);
  const notify = useNotify();
  const refresh = useRefresh();

  const handleClick = async () => {
    if (isLoading) {
      return;
    }

    setIsLoading(true);
    let client = await getAxiosClient();
    await client.put(apiURL + "/attestation_request/" + record.id + "/approve");
    setTimeout(() => {
      setIsLoading(false);
      refresh();
    }, 60_000);
    notify("Transaction for approval is fired");
  };

  return (
    <Tooltip title="Approve">
      <span>
        <Fab
          color="primary"
          aria-label="add"
          size="small"
          disabled={record.approved || isLoading}
          onClick={handleClick}
          sx={{ marginLeft: "1em", marginRight: "1em" }}
        >
          {isLoading ? <CircularProgress color="error" /> : <DoneIcon />}
        </Fab>
      </span>
    </Tooltip>
  );
};

const DisableEditButton = () => {
  const record = useRecordContext<AttestationRequsts>();
  return <EditButton disabled={record.approved} />;
};

const RevokeButton = () => {
  const record = useRecordContext<AttestationRequsts>();
  const [isLoading, setIsLoading] = useState(false);
  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
  const notify = useNotify();
  const refresh = useRefresh();

  const handleClick = async () => {
    if (isLoading) {
      return;
    }
    setIsLoading(true);
    let client = await getAxiosClient();

    await client.put(apiURL + "/attestation_request/" + record.id + "/revoke");
    setTimeout(() => {
      setIsLoading(false);
      refresh();
      notify("Transaction is finished!");
    }, 60_000);
    notify("Transaction for revokation is fired.");
  };

  return (
    <Tooltip title="Revoke">
      <span>
        <Fab
          color="error"
          aria-label="revoke"
          size="small"
          disabled={!record.approved || record.revoked || isLoading}
          onClick={handleClick}
          sx={{ marginLeft: "1em", marginRight: "1em" }}
        >
          {isLoading ? <CircularProgress /> : <RemoveIcon />}
        </Fab>
      </span>
    </Tooltip>
  );
};

const URLField = ({ baseURL }: { source: string; baseURL: string }) => {
  const record = useRecordContext<AttestationRequsts>();
  let ctype = record.ctype_hash;

  if (!ctype.startsWith("kilt:ctype:")) {
    ctype = `kilt:ctype:${ctype}` as ICType["$id"];
  }

  return <a href={`${baseURL}${ctype}`}>{ctype}</a>;
};

export const AttestationList = () => {
  return (
    <List>
      <Datagrid expand={ExpandAttestation}>
        <TextField source="id" />
        <DateField source="created_at" />
        <DateField source="updated_at" />
        <DateField source="approved_at" />
        <DateField source="revoked_at" />
        <TextField source="tx_state" />
        <URLField
          source="ctype_hash"
          baseURL="https://ctypehub.galaniprojects.de/ctype/"
        />
        {isUserAdmin() && <ApproveButton />}
        {isUserAdmin() && <RevokeButton />}
        <DisableEditButton />
      </Datagrid>
    </List>
  );
};
