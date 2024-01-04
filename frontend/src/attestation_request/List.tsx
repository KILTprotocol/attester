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
import DownloadIcon from "@mui/icons-material/Download";
import BookmarkAddedIcon from '@mui/icons-material/BookmarkAdded';
import { ICType } from "@kiltprotocol/sdk-js";

import { AttestationRequest } from "../types";
import { useCallback, useState } from "react";
import { getAxiosClient } from "../dataProvider";
import {
  apiWindow,
  requestAttestation,
  useCompatibleExtensions,
} from "../session";
import { isUserAdmin } from "../utils";

const ExpandAttestation = () => {
  const record = useRecordContext<AttestationRequest>();
  const [theme, _] = useTheme();
  return (
    <ReactJson
      theme={theme === "dark" ? "colors" : "bright:inverted"}
      src={record.credential.claim.contents}
    />
  );
};

const DisableEditButton = () => {
  const record = useRecordContext<AttestationRequest>();
  return <EditButton disabled={record.approved} />;
};

const RevokeApproveButton = () => {
  const record = useRecordContext<AttestationRequest>();
  const [isLoading, setIsLoading] = useState(false);
  const apiURL = import.meta.env.VITE_SIMPLE_REST_URL;
  const notify = useNotify();
  const refresh = useRefresh();

  const handleClick = useCallback(async (url: string) => {
    if (isLoading) {
      return;
    }
    setIsLoading(true);
    let client = await getAxiosClient();

    await client.put(url);
    setTimeout(() => {
      setIsLoading(false);
      refresh();
      notify("Transaction is finished!");
    }, 60_000);
    notify("Transaction is fired.");
  }, []);

  return (
    <>
      {!record.approved || !record.approved_at ? (
        <>
          <Tooltip title="Approve">
            <span>
              <Fab
                color="primary"
                aria-label="add"
                size="small"
                disabled={record.approved || isLoading}
                onClick={() =>
                  handleClick(
                    apiURL + "/attestation_request/" + record.id + "/approve"
                  )
                }
                sx={{ marginRight: "1em" }}
              >
                {isLoading ? <CircularProgress color="error" /> : <DoneIcon />}
              </Fab>
            </span>
          </Tooltip>
          <Tooltip title="Mark Approve">
            <span>
              <Fab
                color="inherit"
                aria-label="add"
                size="small"
                disabled={record.approved || isLoading}
                onClick={() =>
                  handleClick(
                    apiURL + "/attestation_request/" + record.id + "/mark_approve"
                  )
                }
                sx={{ marginLeft: "1em", }}
              >
                {isLoading ? <CircularProgress color="error" /> : <BookmarkAddedIcon />}
              </Fab>
            </span>
          </Tooltip>
        </>
      ) : (
        <Tooltip title="Revoke">
          <span>
            <Fab
              color="error"
              aria-label="revoke"
              size="small"
              disabled={!record.approved || record.revoked || isLoading}
              onClick={() =>
                handleClick(
                  apiURL + "/attestation_request/" + record.id + "/revoke"
                )
              }
              sx={{ marginRight: "1em" }}
            >
              {isLoading ? <CircularProgress /> : <RemoveIcon />}
            </Fab>
          </span>
        </Tooltip>
      )
      }
    </>
  );
};

const DownloadCredential = () => {
  const record = useRecordContext<AttestationRequest>();
  const [isLoading, setIsLoading] = useState(false);
  const { kilt } = apiWindow;
  const notify = useNotify();
  const { extensions } = useCompatibleExtensions();
  const hasExtension = extensions.length > 0;

  const handleClick = useCallback(async () => {
    setIsLoading(true);
    const extension = "sporran";

    try {
      await requestAttestation(kilt[extension], record.id);
    } catch (e) {
      console.error(e);
      notify("Could not request Credential", { type: "error" });
    }
    notify("Credential is downloaded into wallet");
    setIsLoading(false);
  }, [kilt]);

  return (
    <>
      {hasExtension && (
        <Tooltip title="Download">
          <Fab
            color="success"
            aria-label="download"
            size="small"
            disabled={!record.approved_at}
            onClick={handleClick}
            sx={{ marginLeft: "1em", marginRight: "1em" }}
          >
            {isLoading ? <CircularProgress /> : <DownloadIcon />}
          </Fab>
        </Tooltip>
      )}
    </>
  );
};

const URLField = ({ baseURL }: { source: string; baseURL: string }) => {
  const record = useRecordContext<AttestationRequest>();
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
        <TextField source="claimer" />
        <DateField source="created_at" />
        <DateField source="updated_at" />
        <DateField source="approved_at" />
        <DateField source="revoked_at" />
        <TextField source="tx_state" />
        <URLField
          source="ctype_hash"
          baseURL="https://ctypehub.galaniprojects.de/ctype/"
        />
        {isUserAdmin() && <RevokeApproveButton />}
        <DownloadCredential />
        <DisableEditButton />
      </Datagrid>
    </List>
  );
};
