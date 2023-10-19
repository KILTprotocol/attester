import {
  Create,
  SaveButton,
  SimpleForm,
  Toolbar,
  useNotify
} from "react-admin";
import TextField from "@mui/material/TextField";
import {
  ICType,
  IClaimContents,
  Claim,
  DidUri,
  Credential as KiltCredential
} from "@kiltprotocol/sdk-js";
import { useState } from "react";
import ReactJson, { InteractionProps } from "react-json-view";
import { fetchCType } from "../utils";

//TODO:fix
function getDefaultEntryForType({ type }: { type: string }) {
  if (type === "string") {
    return "";
  }
  if (type === "boolean") {
    return false;
  }
  if (type === "number" || type === "integer") {
    return 0;
  }
}

export const AttestationCreate = () => {
  // states
  const [ctypeHash, setCtypeHash] = useState<String>("");
  const [claimer, setClaimer] = useState<String>("");
  const [ctype, setCtypeDetails] = useState<ICType>();

  const [claimContent, setClaimContent] = useState<IClaimContents>();

  // hooks
  const notify = useNotify();

  //callbacks
  const handleSelectedCtype = async (ctype: string) => {
    setCtypeHash(ctype);
    try {
      const ctypeDetails = await fetchCType(ctype as any);
      const claimContent: any = {};
      Object.entries(ctypeDetails.cType.properties).map(
        ([key, type]) =>
          (claimContent[key] = getDefaultEntryForType(type as any))
      );
      setCtypeDetails(ctypeDetails.cType);
      setClaimContent(claimContent);
    } catch {
      setClaimContent(undefined);
      notify("CType does not exists");
    }
  };

  const onEdit = (data: InteractionProps) => {
    setClaimContent(data.updated_src as IClaimContents);
  };

  const transformData = () => {
    if (!ctype || !claimContent) {
      return false;
    }

    try {
      const claim = Claim.fromCTypeAndClaimContents(
        ctype,
        claimContent,
        claimer as DidUri
      );

      const credential = KiltCredential.fromClaim(claim);

      return {
        claimer,
        ctype_hash: ctypeHash,
        credential: credential
      };
    } catch {
      notify("Ctype Verification failed");
    }
  };

  //Elements
  const CustomToolBar = (props: any) => {
    return (
      <Toolbar {...props}>
        <SaveButton
          alwaysEnable={claimer !== "" && claimContent !== undefined}
          label="Save"
        />
      </Toolbar>
    );
  };

  return (
    <Create transform={transformData} redirect="list">
      <SimpleForm toolbar={<CustomToolBar />}>
        <TextField
          value={ctypeHash}
          label="Ctype Hash"
          variant="outlined"
          fullWidth
          onChange={(e) => handleSelectedCtype(e.target.value)}
          required
        />
        <TextField
          value={claimer}
          label="Claimer"
          variant="outlined"
          fullWidth
          onChange={(e) => setClaimer(e.target.value)}
          required
        />
        {claimContent && (
          <ReactJson
            src={claimContent}
            onEdit={onEdit}
            name="Claim"
            validationMessage="Claim Verification failed"
          />
        )}
      </SimpleForm>
    </Create>
  );
};
