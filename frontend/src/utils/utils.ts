import { ICType, CType, connect } from '@kiltprotocol/sdk-js'
import authProvider from '../api/authProvider'

export async function fetchCType(ctypeId: ICType['$id']): Promise<CType.ICTypeDetails> {
  await connect(getKiltEndpoint())
  return CType.fetchFromChain(ctypeId)
}

export function isUserAdmin() {
  const role = authProvider.getRole()
  return role === 'admin'
}

export function storeEndpoints(endpoints: Array<string>) {
  const authorizeUrl = endpoints[0];
  const kiltEndpoint = endpoints[1];
  const backendUrl = endpoints[2];

  localStorage.setItem("authorizeUrl", authorizeUrl);
  localStorage.setItem("kiltEndpoint", kiltEndpoint);
  localStorage.setItem("backendUrl", backendUrl);
}

export function getBackendUrl(): string {
  return localStorage.getItem("backendUrl") || ""
}

export function getKiltEndpoint(): string {
  return localStorage.getItem("kiltEndpoint") || ""
}

export function getAuthorizeUrl(): string {
  return localStorage.getItem("authorizeUrl") || ""
}
