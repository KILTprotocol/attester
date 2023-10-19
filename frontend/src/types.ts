import { DidUri, ICType, ICredential } from "@kiltprotocol/sdk-js";
import { UUID } from "crypto";

Date.now();

export interface AttestationRequsts {
  approved: boolean;
  revoked: boolean;
  claimer: DidUri;
  created_at: string;
  credential: ICredential;
  ctype_hash: ICType["$id"];
  deleted_at?: string;
  id: UUID;
  updated_at?: string;
}