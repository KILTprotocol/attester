import {
  List,
  Datagrid,
  TextField,
  DateField,
  useRecordContext,
  EditButton,
  useNotify
} from "react-admin";
import ReactJson from "react-json-view";
import Fab from "@mui/material/Fab";
import DoneIcon from "@mui/icons-material/Done";
import RemoveIcon from "@mui/icons-material/Remove";
import CircularProgress from "@mui/material/CircularProgress";
import Tooltip from "@mui/material/Tooltip";

import axios from "axios";
import { AttestationRequsts } from "../types";
import { useState } from "react";
import { isUserAdmin } from "../utils";

const ExpandAttestation = () => {
  const record = useRecordContext<AttestationRequsts>();
  return <ReactJson src={record.credential.claim.contents} />;
};

const ApproveButton = () => {
  const record = useRecordContext<AttestationRequsts>();
  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
  const [isLoading, setIsLoading] = useState(false);
  const notify = useNotify();

  const handleClick = async () => {
    if (isLoading) {
      return;
    }

    setIsLoading(true);
    const data = { credential: record.credential, approved: true };
    await axios.put(
      apiURL + "/attestation_request/" + record.id + "/approve",
      data
    );
    setIsLoading(false);
    notify("Attestation is approved");
  };

  return (
    <Tooltip title="Approve">
      <Fab
        color="primary"
        aria-label="add"
        size="small"
        disabled={record.approved}
        onClick={handleClick}
        sx={{ marginLeft: "1em", marginRight: "1em" }}
      >
        {isLoading ? <CircularProgress color="info" /> : <DoneIcon />}
      </Fab>
    </Tooltip>
  );
};

const RevokeButton = () => {
  const record = useRecordContext<AttestationRequsts>();
  const [isLoading, setIsLoading] = useState(false);
  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
  const notify = useNotify();

  const handleClick = async () => {
    if (isLoading) {
      return;
    }
    setIsLoading(true);
    const data = { credential: record.credential, approved: true };
    await axios.put(
      apiURL + "/attestation_request/" + record.id + "/revoke",
      data
    );
    setIsLoading(false);
    notify("Attestation is revoked");
  };

  return (
    <Tooltip title="Revoke">
      <Fab
        color="error"
        aria-label="revoke"
        size="small"
        disabled={!record.approved || record.revoked}
        onClick={handleClick}
        sx={{ marginLeft: "1em", marginRight: "1em" }}
      >
        <RemoveIcon />
        {isLoading && <CircularProgress />}
      </Fab>
    </Tooltip>
  );
};

const URLField = ({ baseURL }: { source: string; baseURL: string }) => {
  const record = useRecordContext<AttestationRequsts>();

  return <a href={baseURL + record.ctype_hash}>{record.ctype_hash}</a>;
};

export const AttestationList = () => {
  return (
    <List>
      <Datagrid expand={ExpandAttestation}>
        <TextField source="id" />
        <DateField source="created_at" />
        <DateField source="updated_at" />
        <URLField
          source="ctype_hash"
          baseURL="https://ctypehub.galaniprojects.de/ctype/"
        />
        {isUserAdmin() && <ApproveButton />}
        {isUserAdmin() && <RevokeButton />}
        <EditButton />
      </Datagrid>
    </List>
  );
};
