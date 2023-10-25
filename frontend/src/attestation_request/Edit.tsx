import {
  DateInput,
  Edit,
  Identifier,
  RaRecord,
  SaveButton,
  SimpleForm,
  TextInput,
  Toolbar,
  useNotify,
  useRecordContext
} from "react-admin";
import ReactJson, { InteractionProps } from "react-json-view";
import Typography from "@mui/material/Typography";
import {
  Claim,
  IClaim,
  IClaimContents,
  Credential as KiltCredentials
} from "@kiltprotocol/sdk-js";
import { useState } from "react";
import { fetchCType } from "../utils";
import { AttestationRequsts } from "../types";

export const AttestationEdit = () => {
  //states
  const [updatedClaim, setUpdatedClaim] = useState<IClaim>();
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // hooks
  const notify = useNotify();

  // callbacks
  const transformData = (previous_data: RaRecord<Identifier>) => {
    const updatedCredential = updatedClaim
      ? KiltCredentials.fromClaim(updatedClaim)
      : previous_data.credential;

    return { claim: updatedCredential };
  };

  // elements
  const EditClaim = () => {
    const record = useRecordContext<AttestationRequsts>();
    const onEdit = async (data: InteractionProps) => {
      setIsLoading(true);
      let ctypeDetails = await fetchCType(record.ctype_hash);
      try {
        const claim = Claim.fromCTypeAndClaimContents(
          ctypeDetails.cType,
          data.updated_src as IClaimContents,
          record.claimer
        );
        setUpdatedClaim(claim);
        notify("Json Updated");
      } catch (e) {
        notify("Json Verification failed", { type: "error" });
        return false;
      }
      setIsLoading(false);
    };

    return (
      <div>
        <Typography variant="inherit">Claim Content</Typography>
        <ReactJson
          src={
            updatedClaim !== undefined
              ? updatedClaim.contents
              : record.credential.claim.contents
          }
          onEdit={!record.approved ? onEdit : false}
          name="Claim"
          validationMessage="Claim Verification failed"
        />
      </div>
    );
  };

  const CustomToolBar = (props: any) => {
    const record = useRecordContext<AttestationRequsts>();

    return (
      <Toolbar {...props}>
        <SaveButton
          {...props}
          label="Save"
          disabled={isLoading || record.approved}
        />
      </Toolbar>
    );
  };

  return (
    <Edit transform={transformData}>
      <SimpleForm toolbar={<CustomToolBar />}>
        <TextInput
          variant="outlined"
          disabled
          label="Id"
          source="id"
          fullWidth
        />
        <TextInput variant="outlined" disabled source="claimer" fullWidth />
        <DateInput variant="outlined" disabled source="created_at" fullWidth />
        <TextInput variant="outlined" disabled source="ctype_hash" fullWidth />
        <EditClaim />
      </SimpleForm>
    </Edit>
  );
};
