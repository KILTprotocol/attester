import { ICType, CType, connect } from "@kiltprotocol/sdk-js";

export async function fetchCType(
  ctypeId: ICType["$id"]
): Promise<CType.ICTypeDetails> {
  await connect(import.meta.env.VITE_WSS_ENDPOINT);
  return CType.fetchFromChain(ctypeId);
}
